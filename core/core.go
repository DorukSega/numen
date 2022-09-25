// Package core is a package which encloses enums under "core."
package core

import (
	"errors"
	"fmt"
	"log"
	"math"
	"numen/enums"
)

var TokenIdMap = map[enums.TokenIDs]string{
	enums.END:       "end",
	enums.IF:        "if",
	enums.IFF:       "iff",
	enums.ELSE:      "else",
	enums.WHILE:     "while",
	enums.DO:        "do",
	enums.TFUNCTION: "fun",
	enums.AS:        "as",
	enums.LET:       "let",
}

var NTypeTokenMap = map[enums.TokenIDs]string{
	enums.TINT:     "int",
	enums.TFLOAT:   "float",
	enums.TSTRING:  "string",
	enums.TBOOLEAN: "bool",
}

var NTypeMap = map[enums.TokenIDs]enums.TokenIDs{ // which type takes what
	enums.TINT:     enums.INT,
	enums.TFLOAT:   enums.FLOAT,
	enums.TSTRING:  enums.STRING,
	enums.TBOOLEAN: enums.BOOL,
}

var BuiltinFunctions = []string{
	"+",
	"-",
	"/",
	"*",
	"%",
	"==",
	"!=",
	">",
	"<",
	">=",
	"<=",
	"print",
	"swap",
	"drop", //TODO pop?
	"copy",
	"max",
	"min",
	"ret",
	"rot",
	"carry",
	"is",
}

var BlockMakers = []enums.TokenIDs{ // keywords that create new blocks
	enums.IF,
	enums.WHILE,
}

type Token struct { // token to hold parsed info
	Id    enums.TokenIDs // type
	Value any
}

type Stack []Token
type Heap map[string]Token

type Block struct {
	Name       string
	Stack      Stack
	Heap       Heap
	Parameters Stack
}

type FileBlockMap map[string]*Block

func SwapLast(h *Stack) {
	i, j := len(*h)-1, len(*h)-2
	(*h)[i], (*h)[j] = (*h)[j], (*h)[i]
}

func Push(h *Stack, item ...Token) {
	*h = append(*h, item...)
}

func Pop(h *Stack) (Token, error) {
	old := *h
	n := len(old)
	if n == 0 {
		return *new(Token), errors.New("stack is empty")
	}
	x := old[n-1]
	*h = old[0 : n-1]
	return x, nil
}

func StartsWithStringQuote(str string) int {
	if str[0] == '"' { // "
		return 1
	} else if str[0] == '\'' { // '
		return 2
	}
	return 0
}

func EndsWithStringQuote(str string) int {
	if str[len(str)-1] == '"' && str[len(str)-2] != '\\' { // "
		return 1
	} else if str[len(str)-1] == '\'' && str[len(str)-2] != '\\' { // "
		return 2
	}
	return 0
}

func MapGetValue[K comparable, V comparable](m map[K]V, key K) (V, bool) {
	for Key, Value := range m {
		if Key == key {
			return Value, true
		}
	}
	return *new(V), false
}

func HeapGetValue(m Heap, key string) (Token, bool) {
	for Key, Value := range m {
		if Key == key {
			return Value, true
		}
	}
	return *new(Token), false
}

func MapGetKey[K comparable, V comparable](m map[K]V, value V) (K, bool) {
	for Key, Value := range m {
		if Value == value {
			return Key, true
		}
	}
	return *new(K), false
}

func MapContainsValue[K comparable, V comparable](m map[K]V, value V) bool {
	for _, Value := range m {
		if Value == value {
			return true
		}
	}
	return false
}

func MapContainsKey[K comparable, V comparable](m map[K]V, key K) bool {
	for Key, _ := range m {
		if Key == key {
			return true
		}
	}
	return false
}

func ArrayContains[K comparable](a []K, value K) bool {
	for _, Value := range a {
		if Value == value {
			return true
		}
	}
	return false
}

func StackReverse(a *Stack) {
	for i, j := 0, len(*a)-1; i < j; i, j = i+1, j-1 {
		(*a)[i], (*a)[j] = (*a)[j], (*a)[i]
	}
}

func StackReversRet(a Stack) Stack {
	for i, j := 0, len(a)-1; i < j; i, j = i+1, j-1 {
		a[i], a[j] = a[j], a[i]
	}
	return a
}

func SafeTop(h Stack, functionName string) Token {
	n := len(h)
	if n == 0 {
		log.Fatalf("Failed to get top of stack in %v", functionName)
	}
	return h[len(h)-1]
}

func SafeAt(h Stack, at int, functionName string) Token {
	n := len(h)
	if n-1 < at || at < 0 {
		log.Fatalf("Failed to get stack[%v] in %v", at, functionName)
	}
	return h[at]
}

func SafePop(stack *Stack, functionName string) Token {
	item, err := Pop(stack)
	if err != nil {
		log.Fatalf("Failed to pop from stack because %v\nAt %v", err, functionName)
	}
	return item
}

func Bprint(stack *Stack) {
	valid := []enums.TokenIDs{enums.INT, enums.BOOL, enums.STRING, enums.FLOAT}
	item := SafePop(stack, "print")
	if ArrayContains(valid, item.Id) {
		fmt.Printf("%v\n", item.Value) //TODO no new line?
	} else {
		log.Fatalf("%v is not printable", item)
	}
}

func Bret(stack *Stack, parentStack *Stack) {
	Push(parentStack, SafePop(stack, "ret"))
}

func Any2Conv[T any](x any, y any) (T, T, bool) {
	a, ok1 := x.(T)
	b, ok2 := y.(T)
	return a, b, ok1 && ok2
}

func Bplus(stack *Stack) {
	valid := []enums.TokenIDs{enums.INT, enums.FLOAT, enums.STRING}
	second := SafePop(stack, "+")
	first := SafePop(stack, "+")
	if ArrayContains(valid, second.Id) && ArrayContains(valid, first.Id) {
		if s, f, ok := Any2Conv[int](second.Value, first.Value); ok {
			Push(stack, Token{Value: f + s, Id: enums.INT})
		} else if s, f, ok := Any2Conv[float64](second.Value, first.Value); ok {
			Push(stack, Token{Value: f + s, Id: enums.FLOAT})
		} else if s, f, ok := Any2Conv[string](second.Value, first.Value); ok {
			Push(stack, Token{Value: f + s, Id: enums.STRING})
		} else { // forces float, if both are different numbers
			si, _ := second.Value.(int)
			fi, _ := first.Value.(int)
			sf, _ := second.Value.(float64)
			ff, _ := first.Value.(float64)

			var total = (float64(fi) + ff) + (float64(si) + sf)
			Push(stack, Token{Value: total, Id: enums.FLOAT})
		}

	} else {
		log.Fatalf("%v and %v is not add-able", second.Value, first.Value)
	}
}

func Bminus(stack *Stack) {
	valid := []enums.TokenIDs{enums.INT, enums.FLOAT}
	second := SafePop(stack, "-")
	first := SafePop(stack, "-")
	if ArrayContains(valid, second.Id) && ArrayContains(valid, first.Id) {
		if s, f, ok := Any2Conv[int](second.Value, first.Value); ok {
			Push(stack, Token{Value: f - s, Id: enums.INT})
		} else if s, f, ok := Any2Conv[float64](second.Value, first.Value); ok {
			Push(stack, Token{Value: f - s, Id: enums.FLOAT})
		} else { // forces float, if both are different numbers
			si, _ := second.Value.(int)
			fi, _ := first.Value.(int)
			sf, _ := second.Value.(float64)
			ff, _ := first.Value.(float64)

			var total = (float64(fi) + ff) - (float64(si) + sf)
			Push(stack, Token{Value: total, Id: enums.FLOAT})
		}
	} else {
		log.Fatalf("%v and %v is not subtract-able", second.Value, first.Value)
	}
}

func Bmultiply(stack *Stack) {
	valid := []enums.TokenIDs{enums.INT, enums.FLOAT}
	second := SafePop(stack, "*")
	first := SafePop(stack, "*")
	if ArrayContains(valid, second.Id) && ArrayContains(valid, first.Id) {
		if s, f, ok := Any2Conv[int](second.Value, first.Value); ok {
			Push(stack, Token{Value: f * s, Id: enums.INT})
		} else if s, f, ok := Any2Conv[float64](second.Value, first.Value); ok {
			Push(stack, Token{Value: f * s, Id: enums.FLOAT})
		} else { // forces float, if both are different numbers
			si, _ := second.Value.(int)
			fi, _ := first.Value.(int)
			sf, _ := second.Value.(float64)
			ff, _ := first.Value.(float64)

			var total = (float64(fi) + ff) * (float64(si) + sf)
			Push(stack, Token{Value: total, Id: enums.FLOAT})
		}
	} else {
		log.Fatalf("%v and %v is not multiply-able", second.Value, first.Value)
	}
}

func Bdivide(stack *Stack) {
	valid := []enums.TokenIDs{enums.INT, enums.FLOAT}
	second := SafePop(stack, "/")
	first := SafePop(stack, "/")
	if ArrayContains(valid, second.Id) && ArrayContains(valid, first.Id) {
		if s, f, ok := Any2Conv[int](second.Value, first.Value); ok {
			Push(stack, Token{Value: f / s, Id: enums.INT})
		} else if s, f, ok := Any2Conv[float64](second.Value, first.Value); ok {
			Push(stack, Token{Value: f / s, Id: enums.FLOAT})
		} else { // forces float, if both are different numbers
			si, _ := second.Value.(int)
			fi, _ := first.Value.(int)
			sf, _ := second.Value.(float64)
			ff, _ := first.Value.(float64)

			var total = (float64(fi) + ff) / (float64(si) + sf)
			Push(stack, Token{Value: total, Id: enums.FLOAT})
		}
	} else {
		log.Fatalf("%v and %v is not divide-able", second.Value, first.Value)
	}
}

func Bmod(stack *Stack) {
	valid := []enums.TokenIDs{enums.INT, enums.FLOAT}
	second := SafePop(stack, "%")
	first := SafePop(stack, "%")
	if ArrayContains(valid, second.Id) && ArrayContains(valid, first.Id) {
		if s, f, ok := Any2Conv[int](second.Value, first.Value); ok {
			Push(stack, Token{Value: f / s, Id: enums.INT})
		} else if s, f, ok := Any2Conv[float64](second.Value, first.Value); ok {
			Push(stack, Token{Value: math.Mod(f, s), Id: enums.FLOAT})
		} else { // forces float, if both are different numbers
			si, _ := second.Value.(int)
			fi, _ := first.Value.(int)
			sf, _ := second.Value.(float64)
			ff, _ := first.Value.(float64)

			var total = math.Mod(float64(fi)+ff, float64(si)+sf)
			Push(stack, Token{Value: total, Id: enums.FLOAT})
		}
	} else {
		log.Fatalf("%v and %v is not mod-able", second.Value, first.Value)
	}
}

func Bswap(stack *Stack) {
	second := SafePop(stack, "swap")
	first := SafePop(stack, "swap")
	Push(stack, second, first)
}

func Bis(stack *Stack) { // is type
	second := SafePop(stack, "is")
	first := SafePop(stack, "is")

	s, f, _ := Any2Conv[string](second.Value, first.Value)
	if typ, ok := MapGetKey(NTypeTokenMap, s); ok {
		if NTypeMap[typ] == first.Id {
			Push(stack, Token{Id: enums.BOOL, Value: true})
		} else {
			Push(stack, Token{Id: enums.BOOL, Value: false})
		}
	} else if typ, ok := MapGetKey(NTypeTokenMap, f); ok {
		if NTypeMap[typ] == second.Id {
			Push(stack, Token{Id: enums.BOOL, Value: true})
		} else {
			Push(stack, Token{Id: enums.BOOL, Value: false})
		}
	} else {
		log.Fatal("No type to compare using is")
	}
}

func Brot(stack *Stack) {
	third := SafePop(stack, "swap")
	second := SafePop(stack, "swap")
	first := SafePop(stack, "swap")
	Push(stack, first, third, second)
}

func Bdrop(stack *Stack) {
	SafePop(stack, "drop")
}

func Bcopy(stack *Stack) {
	top := SafeTop(*stack, "copy")
	Push(stack, top)
}

func NumMax[T int | float64](x, y T) T {
	if x > y {
		return x
	}
	return y
}

func Bmax(stack *Stack) {
	valid := []enums.TokenIDs{enums.INT, enums.FLOAT}
	second := SafePop(stack, "max")
	first := SafePop(stack, "max")
	if ArrayContains(valid, second.Id) && ArrayContains(valid, first.Id) {
		if s, f, ok := Any2Conv[int](second.Value, first.Value); ok {
			Push(stack, Token{Value: NumMax(f, s), Id: enums.INT})
		} else if s, f, ok := Any2Conv[float64](second.Value, first.Value); ok {
			Push(stack, Token{Value: NumMax(f, s), Id: enums.FLOAT})
		} else { // forces float, if both are different numbers
			si, _ := second.Value.(int)
			fi, _ := first.Value.(int)
			sf, _ := second.Value.(float64)
			ff, _ := first.Value.(float64)

			var total = NumMax(float64(fi)+ff, float64(si)+sf)
			Push(stack, Token{Value: total, Id: enums.FLOAT})
		}
	} else {
		log.Fatalf("%v and %v is not comparable for max", second.Value, first.Value)
	}
}

func NumMin[T int | float64](x, y T) T {
	if x < y {
		return x
	}
	return y
}

func Bmin(stack *Stack) {
	valid := []enums.TokenIDs{enums.INT, enums.FLOAT}
	second := SafePop(stack, "min")
	first := SafePop(stack, "min")
	if ArrayContains(valid, second.Id) && ArrayContains(valid, first.Id) {
		if s, f, ok := Any2Conv[int](second.Value, first.Value); ok {
			Push(stack, Token{Value: NumMin(f, s), Id: enums.INT})
		} else if s, f, ok := Any2Conv[float64](second.Value, first.Value); ok {
			Push(stack, Token{Value: NumMin(f, s), Id: enums.FLOAT})
		} else { // forces float, if both are different numbers
			si, _ := second.Value.(int)
			fi, _ := first.Value.(int)
			sf, _ := second.Value.(float64)
			ff, _ := first.Value.(float64)

			var total = NumMin(float64(fi)+ff, float64(si)+sf)
			Push(stack, Token{Value: total, Id: enums.FLOAT})
		}
	} else {
		log.Fatalf("%v and %v is not comparable for min", second.Value, first.Value)
	}
}

func Bcarry(stack *Stack) {
	tsec := SafeAt(*stack, len(*stack)-2, "carry")
	Push(stack, tsec)
}

func Bequals(stack *Stack) {
	second := SafePop(stack, "equals")
	first := SafePop(stack, "equals")
	Push(stack, Token{Id: enums.BOOL, Value: first.Value == second.Value})
}

func Bnotequals(stack *Stack) {
	second := SafePop(stack, "not equals")
	first := SafePop(stack, "not equals")
	Push(stack, Token{Id: enums.BOOL, Value: first.Value != second.Value})
}

func Bbigger(stack *Stack) {
	valid := []enums.TokenIDs{enums.INT, enums.FLOAT}
	second := SafePop(stack, ">")
	first := SafePop(stack, ">")
	if ArrayContains(valid, second.Id) && ArrayContains(valid, first.Id) {
		if s, f, ok := Any2Conv[int](second.Value, first.Value); ok {
			Push(stack, Token{Value: f > s, Id: enums.BOOL})
		} else if s, f, ok := Any2Conv[float64](second.Value, first.Value); ok {
			Push(stack, Token{Value: f > s, Id: enums.BOOL})
		} else { // forces float, if both are different numbers
			si, _ := second.Value.(int)
			fi, _ := first.Value.(int)
			sf, _ := second.Value.(float64)
			ff, _ := first.Value.(float64)

			var total = float64(fi)+ff > float64(si)+sf
			Push(stack, Token{Value: total, Id: enums.BOOL})
		}
	} else {
		log.Fatalf("%v and %v is not comparable for >", second.Value, first.Value)
	}
}

func Bsmaller(stack *Stack) {
	valid := []enums.TokenIDs{enums.INT, enums.FLOAT}
	second := SafePop(stack, "<")
	first := SafePop(stack, "<")
	if ArrayContains(valid, second.Id) && ArrayContains(valid, first.Id) {
		if s, f, ok := Any2Conv[int](second.Value, first.Value); ok {
			Push(stack, Token{Value: f < s, Id: enums.BOOL})
		} else if s, f, ok := Any2Conv[float64](second.Value, first.Value); ok {
			Push(stack, Token{Value: f < s, Id: enums.BOOL})
		} else { // forces float, if both are different numbers
			si, _ := second.Value.(int)
			fi, _ := first.Value.(int)
			sf, _ := second.Value.(float64)
			ff, _ := first.Value.(float64)

			var total = float64(fi)+ff < float64(si)+sf
			Push(stack, Token{Value: total, Id: enums.BOOL})
		}
	} else {
		log.Fatalf("%v and %v is not comparable for <", second.Value, first.Value)
	}
}

func Bbiggerequals(stack *Stack) {
	valid := []enums.TokenIDs{enums.INT, enums.FLOAT}
	second := SafePop(stack, ">=")
	first := SafePop(stack, ">=")
	if ArrayContains(valid, second.Id) && ArrayContains(valid, first.Id) {
		if s, f, ok := Any2Conv[int](second.Value, first.Value); ok {
			Push(stack, Token{Value: f >= s, Id: enums.BOOL})
		} else if s, f, ok := Any2Conv[float64](second.Value, first.Value); ok {
			Push(stack, Token{Value: f >= s, Id: enums.BOOL})
		} else { // forces float, if both are different numbers
			si, _ := second.Value.(int)
			fi, _ := first.Value.(int)
			sf, _ := second.Value.(float64)
			ff, _ := first.Value.(float64)

			var total = float64(fi)+ff >= float64(si)+sf
			Push(stack, Token{Value: total, Id: enums.BOOL})
		}
	} else {
		log.Fatalf("%v and %v is not comparable for >=", second.Value, first.Value)
	}
}

func Bsmallerequals(stack *Stack) {
	valid := []enums.TokenIDs{enums.INT, enums.FLOAT}
	second := SafePop(stack, "<=")
	first := SafePop(stack, "<=")
	if ArrayContains(valid, second.Id) && ArrayContains(valid, first.Id) {
		if s, f, ok := Any2Conv[int](second.Value, first.Value); ok {
			Push(stack, Token{Value: f <= s, Id: enums.BOOL})
		} else if s, f, ok := Any2Conv[float64](second.Value, first.Value); ok {
			Push(stack, Token{Value: f <= s, Id: enums.BOOL})
		} else { // forces float, if both are different numbers
			si, _ := second.Value.(int)
			fi, _ := first.Value.(int)
			sf, _ := second.Value.(float64)
			ff, _ := first.Value.(float64)

			var total = float64(fi)+ff <= float64(si)+sf
			Push(stack, Token{Value: total, Id: enums.BOOL})
		}
	} else {
		log.Fatalf("%v and %v is not comparable for <=", second.Value, first.Value)
	}
}
