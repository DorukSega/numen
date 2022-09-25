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
		(*fileBlockMap)[mainBlockName] = &Block{Name: mainBlockName, Stack: Stack{}, Parameters: Stack{}} //init
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

			(*fileBlockMap)[fname] = &Block{Name: fname, Stack: Stack{}, Parameters: Stack{}}
			funcBlock := (*fileBlockMap)[fname]

			// gets parameters
			scanner.Scan() //function name
			for prName := scanner.Text(); prName != TokenIdMap[enums.AS]; _, prName = scanner.Scan(), scanner.Text() {
				if prType, isType := MapGetKey(&NTypeTokenMap, prName); isType {
					Push(&funcBlock.Parameters, Token{Id: prType, Value: prName})
				} else { // is variable
					Push(&funcBlock.Parameters, Token{Id: enums.IDENTIFIER, Value: prName})
				}
			}

			scanner.Scan() // AS

			blockCount := 0
			for tok := scanner.Text(); tok != TokenIdMap[enums.END] || blockCount != 0; _, tok = scanner.Scan(), scanner.Text() {
				if tok == TokenIdMap[enums.IF] || tok == TokenIdMap[enums.WHILE] {
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
		} else if key, ok := MapGetKey(&NTypeTokenMap, word); ok {
			Push(&(*mainBlock).Stack, Token{
				Value: word,
				Id:    key,
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
	fakePStack := &Stack{}
	globalHeap := &Heap{}
	interpret(fileBlockMap, global, global, fakePStack, globalHeap)
	interpret(fileBlockMap, "main", global, fakePStack, globalHeap)
	if len(*fakePStack) != 0 {
		log.Fatal("it is not posible to return from Global or Main")
	}
	fakePStack = nil
	globalHeap = nil
}

func interpret(fileBlockMap *FileBlockMap, functionName string, parentBlockName string, parentContextStack *Stack, globalHeap *Heap) {
	functionBlock, parentBlock := (*fileBlockMap)[functionName], (*fileBlockMap)[parentBlockName]
	functionHeap := Heap{}
	//evaluate function parameters
	interpretedStack := &Stack{} // context is used as live stack
	for _, par := range StackReversRet(functionBlock.Parameters) {
		if MapContainsKey(&NTypeTokenMap, par.Id) {
			popped, err := Pop(parentContextStack)
			if err != nil {
				log.Fatalf("No stack items to get from %v because %v", parentBlockName, err)
			}
			if popped.Id == NTypeMap[par.Id] {
				Push(interpretedStack, popped)
			} else {
				log.Fatalf("parameter's type (%v) that it refers to (%v) and popped item's type (%v) does not match", par.Id, NTypeMap[par.Id], popped.Id)
			}
		} else if par.Id == enums.IDENTIFIER {
			popped, err := Pop(parentContextStack)
			if err != nil {
				log.Fatalf("No stack items to get from %v because %v", parentBlockName, err)
			}
			functionHeap[par.Value.(string)] = popped // inserts to Heap
		}
	}

	StackReverse(interpretedStack)
	*interpretedStack = append(*interpretedStack, (*functionBlock).Stack...)
	interpretStack(fileBlockMap, functionBlock, parentBlock, interpretedStack, parentContextStack, functionHeap, globalHeap)
	interpretedStack = nil
	functionHeap = nil
}

func interpretStack(fileBlockMap *FileBlockMap, functionBlock *Block, parentBlock *Block, interpretedStack *Stack, parentContextStack *Stack, functionHeap Heap, globalHeap *Heap) {
	context := &Stack{}
	for i := 0; i < len(*interpretedStack); i++ {
		item := (*interpretedStack)[i]
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
			identifier := SafePop(context, "let")
			identity, ok := identifier.Value.(string)
			if !ok {
				log.Fatalf("%v is not a identifier in let", identifier)
			}
			value := SafePop(context, "let")
			functionHeap[identity] = value
		} else if MapContainsKey(&NTypeTokenMap, item.Id) {
			identifier := SafePop(context, "decleration")
			identity, ok := identifier.Value.(string)
			if !ok {
				log.Fatalf("%v is not a identifier in type decleration", identifier)
			}
			value := SafePop(context, "decleration")
			if NTypeMap[item.Id] != value.Id {
				log.Fatalf("Type Declaration's %v target type doesn't match %v value's type", NTypeMap[item.Id], value.Id)
			}
			functionHeap[identity] = value
		} else if item.Id == enums.IF || item.Id == enums.IFF {
			condition := SafePop(context, "if")
			if parsedCondition, ok := condition.Value.(bool); ok {
				if parsedCondition { // if
					ifBlock := &Block{Stack: Stack{}, Parameters: Stack{}} // other than Heap, I can't see any use
					ifInterStack := &Stack{}                               // this will have the if's parsed stack
					blockCount := 0
					i++
					ifitem := (*interpretedStack)[i]
					for (ifitem.Id != enums.ELSE && ifitem.Id != enums.END) || blockCount != 0 {
						if ArrayContains(&BlockMakers, ifitem.Id) { // TODO convert this to a block checker function
							blockCount++
						} else if ifitem.Id == enums.END {
							blockCount--
						}
						Push(ifInterStack, ifitem)
						i++
						ifitem = (*interpretedStack)[i]
					}
					i++
					ifitem = (*interpretedStack)[i]
					for (ifitem.Id != enums.END) || blockCount != 0 {
						if ArrayContains(&BlockMakers, ifitem.Id) {
							blockCount++
						} else if ifitem.Id == enums.END {
							blockCount--
						}
						i++
						ifitem = (*interpretedStack)[i]
					}

					interpretStack(fileBlockMap, ifBlock, functionBlock, ifInterStack, parentContextStack, functionHeap, globalHeap)
					ifBlock = nil
					ifInterStack = nil
				} else { // else
					elseBlock := &Block{Stack: Stack{}, Parameters: Stack{}}
					elseInterStack := &Stack{} // this will have the elses's parsed stack
					blockCount := 0
					i++
					elseitem := (*interpretedStack)[i]
					for (elseitem.Id != enums.ELSE && elseitem.Id != enums.END) || blockCount != 0 {
						if ArrayContains(&BlockMakers, elseitem.Id) {
							blockCount++
						} else if elseitem.Id == enums.END {
							blockCount--
						}
						i++
						elseitem = (*interpretedStack)[i]
					}
					i++
					elseitem = (*interpretedStack)[i]
					for (elseitem.Id != enums.END) || blockCount != 0 {
						if ArrayContains(&BlockMakers, elseitem.Id) {
							blockCount++
						} else if elseitem.Id == enums.END {
							blockCount--
						}
						Push(elseInterStack, elseitem)
						i++
						elseitem = (*interpretedStack)[i]
					}
					Push(elseInterStack, elseitem)
					interpretStack(fileBlockMap, elseBlock, functionBlock, elseInterStack, parentContextStack, functionHeap, globalHeap)
					elseBlock = nil
					elseInterStack = nil
				}
			} else {
				log.Fatalf("%v value before 'if' is not a condition", condition)
			}
		} else if item.Id == enums.IDENTIFIER {
			name := item.Value.(string)
			if ok := FileBlockMapContainsKey(fileBlockMap, name); ok {
				interpret(fileBlockMap, name, functionBlock.Name, context, globalHeap)
			} else if fHeapToken, ok := HeapGetValue(&functionHeap, name); ok {
				Push(context, fHeapToken)
			} else if fHeapToken, ok := HeapGetValue(globalHeap, name); ok {
				Push(context, fHeapToken)
			} else {
				Push(context, item)
			}
		} else {
			Push(context, item)
		}
	}
	context = nil

}

//func compile{} // if ever
