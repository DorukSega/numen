package main

import (
	"bufio"
	"fmt"
	"log"
	. "numen/core"
	"numen/enums"
	"os"
	"path/filepath"
	"strconv"
	"strings"
)

// Global Function
const global = "_global"

func main() {
	for _, file := range os.Args[1:] {
		fmt.Printf("Reading: %s \n", file)
		readFile(file)

	}

}

func readFile(filename string) *FileBlockMap {

	dir := filepath.Join(filepath.Dir(os.Args[0]), filename)
	file, err := os.Open(dir)
	if err != nil {
		log.Fatal(err)
	}

	var fileBlockMap = parseFile(file)
	for k, v := range *fileBlockMap {
		fmt.Printf("%s: %v\n\n", k, *v)
	}
	interpretGlobal(fileBlockMap)

	if errc := file.Close(); errc != nil {
		log.Fatal(errc)
	}
	return fileBlockMap
}

func parseFile(file *os.File) *FileBlockMap {
	var fileBlockMap = &FileBlockMap{
		global: &Block{Name: global},
	}
	scanner := bufio.NewScanner(file)
	scanner.Split(bufio.ScanWords) // scans words

	parseBlock(scanner, global, fileBlockMap)
	if err := scanner.Err(); err != nil {
		log.Fatal(err)
	}

	return fileBlockMap
}

func parseBlock(scanner *bufio.Scanner, mainBlockName string, fileBlockMap *FileBlockMap) {
	if (*fileBlockMap)[mainBlockName] == nil {
		(*fileBlockMap)[mainBlockName] = &Block{Name: mainBlockName, Stack: []Token{}, Parameters: []Token{}} //init
	}

	mainBlock := (*fileBlockMap)[mainBlockName]
	for scanner.Scan() {
		word := scanner.Text()
		if StartsWithStringQuote(word) > 0 {
			qtype := StartsWithStringQuote(word)
			word = word[1:]
			for EndsWithStringQuote(word) != qtype {
				scanner.Scan()
				word += " " + scanner.Text()
			}
			word = word[:len(word)-1]

			Push(&(*mainBlock).Stack, Token{
				Value: word,
				Id:    enums.STRING,
			})
		} else if word == TokenIdMap[enums.TFUNCTION] {
			scanner.Scan()
			fname := scanner.Text()
			fmt.Printf("parsing %s\n", fname) // DEBUG

			var funwords string

			(*fileBlockMap)[fname] = &Block{Name: fname, Stack: []Token{}, Parameters: []Token{}}
			funcBlock := (*fileBlockMap)[fname]

			// gets parameters
			scanner.Scan() //function name
			for prName := scanner.Text(); prName != TokenIdMap[enums.AS]; _, prName = scanner.Scan(), scanner.Text() {
				if prType, isType := MapGetValue(&NTypeMap, prName); isType {
					Push(&funcBlock.Parameters, Token{Id: prType, Value: prName})
				} else { // is variable
					Push(&funcBlock.Parameters, Token{Id: enums.IDENTIFIER, Value: prName})
				}
			}

			scanner.Scan() // AS

			blockCount := 0
			for tok := scanner.Text(); tok != TokenIdMap[enums.END] || blockCount != 0; _, tok = scanner.Scan(), scanner.Text() {
				k, _ := MapGetKey(&TokenIdMap, tok)
				if ArrayContains(&BlockDeclerations, k) {
					blockCount++
				} else if tok == TokenIdMap[enums.END] {
					blockCount--
				}
				funwords += " " + tok
			}

			nscanner := bufio.NewScanner(strings.NewReader(funwords))
			nscanner.Split(bufio.ScanWords)
			parseBlock(nscanner, fname, fileBlockMap)
		} else if ArrayContains(&BuiltinFunctions, word) {
			Push(&(*mainBlock).Stack, Token{
				Value: word,
				Id:    enums.FUNCTION,
			})
		} else if key, ok := MapGetKey(&TokenIdMap, word); ok {
			Push(&(*mainBlock).Stack, Token{
				Value: word,
				Id:    key,
			})
		} else if MapContainsKey(&NTypeMap, word) {
			Push(&(*mainBlock).Stack, Token{
				Value: word,
				Id:    enums.TYPE,
			})
		} else if conv, err := strconv.Atoi(word); err == nil {
			Push(&(*mainBlock).Stack, Token{
				Value: conv,
				Id:    enums.INT,
			})
		} else if conv, err := strconv.ParseFloat(word, 64); err == nil {
			Push(&(*mainBlock).Stack, Token{
				Value: conv,
				Id:    enums.FLOAT,
			})
		} else if conv, err := strconv.ParseBool(word); err == nil {
			Push(&(*mainBlock).Stack, Token{
				Value: conv,
				Id:    enums.BOOL,
			})
		} else {
			Push(&(*mainBlock).Stack, Token{
				Value: word,
				Id:    enums.IDENTIFIER,
			})
		}
	}
}

// interprets main and global
func interpretGlobal(fileBlockMap *FileBlockMap) {
	fakePStack := &[]Token{}
	interpret(fileBlockMap, global, fakePStack)
	interpret(fileBlockMap, "main", fakePStack)
	if len(*fakePStack) != 0 {
		log.Fatal("it is not posible to return from Global or Main")
	}
	fakePStack = nil
}

func interpret(fileBlockMap *FileBlockMap, functionName string, parentContextStack *[]Token) {
	functionBlock := (*fileBlockMap)[functionName]
	functionHeap := Heap{}
	//evaluate function parameters
	interpretedStack := &[]Token{} // context is used as live stack
	for _, par := range StackReversRet(functionBlock.Parameters) {
		if par.Id == enums.TYPE {
			popped, err := Pop(parentContextStack)
			if err != nil {
				log.Fatalf("No stack items to get from parent because %v", err)
			}
			if prval, ok := par.Value.(string); popped.Id == NTypeMap[prval] && ok {
				Push(interpretedStack, popped)
			} else {
				log.Fatalf("parameter's type (%v) that it refers to (%v) and popped item's type (%v) does not match", par.Id, NTypeMap[prval], popped.Id)
			}
		} else if par.Id == enums.IDENTIFIER {
			popped, err := Pop(parentContextStack)
			if err != nil {
				log.Fatalf("No stack items to get from parent because %v", err)
			}
			functionHeap[par.Value.(string)] = popped // inserts to Heap
		}
	}
	StackReverse(interpretedStack)
	*interpretedStack = append(*interpretedStack, (*functionBlock).Stack...)

	interpretStack(fileBlockMap, *interpretedStack, parentContextStack, &functionHeap)
	interpretedStack = nil
	functionHeap = nil
}

func interpretStack(fileBlockMap *FileBlockMap, parsedStack []Token, parentContextStack *[]Token, functionHeap *Heap) []Token {
	context := &[]Token{}

	for i := 0; i < len(parsedStack); i++ {
		item := parsedStack[i]

		if item.Id == enums.FUNCTION { // builtin function
			switch item.Value.(string) {
			case "print":
				Bprint(context)
			case "ret":
				Bret(context, parentContextStack)
			case "+":
				Bplus(context)
			case "-":
				Bminus(context)
			case "*":
				Bmultiply(context)
			case "/":
				Bdivide(context)
			case "%":
				Bmod(context)
			case "swap":
				Bswap(context)
			case "drop":
				Bdrop(context)
			case "copy":
				Bcopy(context)
			case "carry":
				Bcarry(context)
			case "rot":
				Brot(context)
			case "max":
				Bmax(context)
			case "min":
				Bmin(context)
			case "==":
				Bequals(context)
			case "!=":
				Bnotequals(context)
			case ">":
				Bbigger(context)
			case "<":
				Bsmaller(context)
			case ">=":
				Bbiggerequals(context)
			case "<=":
				Bsmallerequals(context)
			case "is":
				Bis(context)
			}
		} else if item.Id == enums.LET {
			var BlockStack []Token
			var LetHeap *Heap
			*LetHeap = *functionHeap
			blockCount := 0

			i++ // skip first
			//evaluate function parameters
			var tempStack []Token
			letitem := parsedStack[i]
			for letitem.Id != enums.AS {
				Push(&tempStack, letitem)
				i++
				letitem = parsedStack[i]
			}
			i++ // AS
			letitem = parsedStack[i]
			for _, par := range StackReversRet(tempStack) {
				if par.Id == enums.TYPE {
					popped := SafePop(context, "let")
					if prval, ok := par.Value.(string); popped.Id == NTypeMap[prval] && ok {
						Push(&BlockStack, popped)
					} else {
						log.Fatalf("parameter's type (%v) that it refers to (%v) and popped item's type (%v) does not match", par.Id, NTypeMap[prval], popped.Id)
					}
				} else if par.Id == enums.IDENTIFIER {
					popped := SafePop(context, "let")
					(*LetHeap)[par.Value.(string)] = popped // inserts to Heap
				}
			}
			for (letitem.Id != enums.END) || blockCount != 0 {
				if ArrayContains(&BlockDeclerations, letitem.Id) {
					blockCount++
				} else if letitem.Id == enums.END {
					blockCount--
				}
				Push(&BlockStack, letitem)
				i++
				letitem = parsedStack[i]
			}
			result := interpretStack(fileBlockMap, BlockStack, parentContextStack, LetHeap)
			*context = append(*context, result...)
		} else if item.Id == enums.WHILE {
			var BlockStack []Token
			var CondStack []Token
			blockCount := 0
			i++ // skip first
			whlitem := parsedStack[i]
			for whlitem.Id != enums.DO {
				Push(&CondStack, whlitem)
				i++
				whlitem = parsedStack[i]
			}
			i++
			whlitem = parsedStack[i]
			for (whlitem.Id != enums.END) || blockCount != 0 {
				if ArrayContains(&BlockDeclerations, whlitem.Id) {
					blockCount++
				} else if whlitem.Id == enums.END {
					blockCount--
				}
				Push(&BlockStack, whlitem)
				i++
				whlitem = parsedStack[i]
			}
			var WhileHeap *Heap
			*WhileHeap = *functionHeap
			condres := interpretStack(fileBlockMap, CondStack, parentContextStack, WhileHeap)
			condition := SafePop(&condres, "while")
			var parsedCondition, ok = condition.Value.(bool)

			for ok && parsedCondition {
				result := interpretStack(fileBlockMap, BlockStack, parentContextStack, WhileHeap)
				*context = append(*context, result...)
				//next
				nres := interpretStack(fileBlockMap, CondStack, parentContextStack, WhileHeap)
				ncond := SafePop(&nres, "while")
				parsedCondition, ok = ncond.Value.(bool)
			}
		} else if item.Id == enums.IF || item.Id == enums.IFF {
			var BlockStack []Token
			blockCount := 0
			i++ // skip first
			condition := SafePop(context, "if")
			if parsedCondition, ok := condition.Value.(bool); ok {
				if parsedCondition { // if
					ifitem := parsedStack[i]
					for (ifitem.Id != enums.ELSE && ifitem.Id != enums.END) || blockCount != 0 {
						if ArrayContains(&BlockDeclerations, ifitem.Id) { // TODO convert this to a block checker function
							blockCount++
						} else if ifitem.Id == enums.END {
							blockCount--
						}
						Push(&BlockStack, ifitem)
						i++
						ifitem = parsedStack[i]
					}
					i++
					ifitem = parsedStack[i]
					for (ifitem.Id != enums.END) || blockCount != 0 {
						if ArrayContains(&BlockDeclerations, ifitem.Id) {
							blockCount++
						} else if ifitem.Id == enums.END {
							blockCount--
						}
						i++
						ifitem = parsedStack[i]
					}
				} else { // else
					elseitem := parsedStack[i]
					for (elseitem.Id != enums.ELSE && elseitem.Id != enums.END) || blockCount != 0 {
						if ArrayContains(&BlockDeclerations, elseitem.Id) {
							blockCount++
						} else if elseitem.Id == enums.END {
							blockCount--
						}
						i++
						elseitem = parsedStack[i]
					}
					i++
					elseitem = parsedStack[i]
					for (elseitem.Id != enums.END) || blockCount != 0 {
						if ArrayContains(&BlockDeclerations, elseitem.Id) {
							blockCount++
						} else if elseitem.Id == enums.END {
							blockCount--
						}
						Push(&BlockStack, elseitem)
						i++
						elseitem = parsedStack[i]
					}
				}
			} else {
				log.Fatalf("%v value before 'if' is not a condition", condition)
			}
			result := interpretStack(fileBlockMap, BlockStack, parentContextStack, functionHeap)
			*context = append(*context, result...)
		} else if item.Id == enums.IDENTIFIER {
			name := item.Value.(string)
			if ok := FileBlockMapContainsKey(fileBlockMap, name); ok {
				interpret(fileBlockMap, name, context)
			} else if value, ok := HeapGetValue(functionHeap, name); ok { // TODO add a global const values
				Push(context, value)
			} else { // TODO add a global const values
				Push(context, item)
			}
		} else {
			Push(context, item)
		}
	}

	return *context
	//	context = nil
}

//func compile{} // if ever
