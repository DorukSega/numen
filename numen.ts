import * as fs from 'fs';
import * as path from 'path';
import {CONNREFUSED} from "dns";

declare global {
    interface Array<T> {
        top(): T;
    }

    interface Map<K, V> {
        getKey(value: V): K | undefined;
    }
}

Map.prototype.getKey = function (value) {
    this.forEach((val, key) => {
        if (value === val)
            return key;
    })
}

Array.prototype.top = function () {
    return this[this.length - 1];
}

const N_debug = false;

enum TOKENS {
    FUNCTION, // BUILTIN FUNCTION
    NUMBER, // JS Number
    INTEGER, // Integer
    FLOAT, // Float
    DOUBLE,
    STRING, //String
    KEYWORD, // KEYWORD
    IDENTIFIER, // potential identifier
    T_NUMBER, // type declarations
    T_INT, //
    T_FLOAT,
    T_DOUBLE,
    T_STRING,
    T_FUNCTION,
    END,
    IF,
    ELIF,
    ELSE,
    AS,
}


const Keywords = new Map<TOKENS, string>([
    [TOKENS.END, "end"],
    [TOKENS.IF, "if"],
    [TOKENS.ELSE, "else"],
    [TOKENS.T_FUNCTION, "fun"],
    [TOKENS.AS, "as"],
]);

//builtin functions - operands
const Functions = new Map<string, TOKENS>([
    ["+", TOKENS.FUNCTION],
    ["-", TOKENS.FUNCTION],
    ["/", TOKENS.FUNCTION],
    ["*", TOKENS.FUNCTION],
    ["%", TOKENS.FUNCTION],
    ["print", TOKENS.FUNCTION],
    ["swap", TOKENS.FUNCTION],
    ["drop", TOKENS.FUNCTION], //TODO pop?
    ["copy", TOKENS.FUNCTION],
    ["max", TOKENS.FUNCTION],
    ["min", TOKENS.FUNCTION],
    ["ret", TOKENS.FUNCTION],
]);

//builtin types
const Types = new Map<string, TOKENS>([
    ["num", TOKENS.NUMBER],
    ["int", TOKENS.T_INT],
    ["float", TOKENS.T_FLOAT],
    ["double", TOKENS.T_DOUBLE],
    ["string", TOKENS.STRING],
])
type possibleTypes = string | number | boolean | Array<possibleTypes>;

interface Token {
    type: TOKENS,
    value: possibleTypes
}


type Stack = Array<Token>;
type Block = Map<string, Token>;

class n_Function {
    params: Stack;
    func_stack: Stack;
    name: string;

    constructor(name: string) {
        this.params = new Array<Token>();
        this.func_stack = new Array<Token>();
        this.name = name;
    }
}

// Stack Map
let m_Stack = new Map<string, n_Function>([
    ["_global", new n_Function("_global")],
]);
// Block Map
let m_Block = new Map<string, Block>([
    ["_global", new Map<string, Token>([
        ["version", {type: TOKENS.STRING, value: "alpha"}],
    ])],
]);


//TODO get this more parsed
let file = process.argv[process.argv.length - 1];

//main
processFile(file);


function processFile(filename: string) {
    console.log("Reading file " + filename);
    let data = fs.readFileSync(path.join(__dirname, filename), {encoding: 'utf8', flag: 'r'},);

    let wsarr = data.replace("\r", "").split(/[\s\n]+/);
    //console.log(wsarr)

    parseFile(wsarr);
    execute(m_Stack.get("_global")!);
    execute(m_Stack.get("main")!, m_Stack.get("_global")!);

}

function parseWord(word: string): Token {
    if (Functions.has(word)) {
        return {
            type: TOKENS.FUNCTION,
            value: word
        };
    } else if (Types.has(word)) {
        return {
            type: Types.get(word)!,
            value: word
        };
    } else if (!isNaN(Number(word))) {
        return {
            type: TOKENS.NUMBER,
            value: Number(word)
        };
    } else {
        return {
            type: TOKENS.IDENTIFIER,
            value: word
        };
    }
}

function parseFile(wsarr: string[]) {

    let g_stack = m_Stack.get("_global")!.func_stack;
    for (let i = 0; i < wsarr.length; i++) {
        let word = wsarr[i];
        if (stringStartsWith(word)) {
            let qtype = stringStartsWith(word);
            word = word.slice(1);

            while (stringEndsWith(word) != qtype)
                word += " " + wsarr[++i];

            word = word.slice(0, -1);

            g_stack.push({
                type: TOKENS.STRING,
                value: word
            });
        } else if (word === Keywords.get(TOKENS.T_FUNCTION)) {
            let name = wsarr[++i];
            console.log("new function " + name) // DEBUG
            let wsf = new Array<string>;
            let n_params = new Array<Token>();

            while (wsarr[i + 1] != Keywords.get(TOKENS.AS)) {
                let pr_name = wsarr[++i];
                let pr_type = Types.get(pr_name);
                if (!pr_type)
                    throw pr_name + " is not a type";
                n_params.push({
                    type: pr_type,
                    value: pr_name
                });
            }
            i++; //as

            while (wsarr[i + 1] != Keywords.get(TOKENS.END)) {
                wsf.push(wsarr[++i]);
            }
            i++; // end

            let func = new n_Function(name);
            func.params = n_params;
            m_Stack.set(name, func);
            m_Block.set(func.name, new Map<string, Token>());
            parseStack(wsf, m_Stack.get(name)!.func_stack);
        } else {
            g_stack.push(parseWord(word));
        }
    }

    // debug print
    if (g_stack.length && N_debug) {
        console.log("\nGlobal:");
        g_stack.forEach(it => {
            process.stdout.write(` { ${it.value} } `);
        })
        console.log("\n")
    }
}

//parses a indiviual stack
function parseStack(wsarr: string[], stack: Stack) {

    for (let i = 0; i < wsarr.length; i++) {
        let word = wsarr[i];

        if (stringStartsWith(word)) {
            let qtype = stringStartsWith(word);
            word = word.slice(1);

            while (stringEndsWith(word) != qtype)
                word += " " + wsarr[++i];

            word = word.slice(0, -1);

            stack.push({
                type: TOKENS.STRING,
                value: word
            });
        } else {
            stack.push(parseWord(word));
        }
    }

    // debug print
    if (stack.length && N_debug) {
        console.log("\nFunc:");
        stack.forEach(it => {
            process.stdout.write(` { ${it.value} } `);
        })
    }
}


function stringStartsWith(str: string): number {
    if (str.startsWith('"')) // "
        return 1;
    else if (str.startsWith("'")) // '
        return 2;
    return 0;
}

function stringEndsWith(str: string): number {
    if (str.endsWith('"') && !str.endsWith('\\"')) // "
        return 1;
    else if (str.endsWith("'") && !str.endsWith("\\'")) // '
        return 2;
    return 0;
}


function execute(func: n_Function, g_func: n_Function = func) {

    let block = m_Block.get(func.name);
    let g_block = m_Block.get(g_func.name);

    if (!block)
        throw func.name + "'s block is undefined";
    if (!g_block)
        throw g_func.name + "'s block is undefined";

    let stack = func.func_stack;
    let g_stack = g_func.func_stack;

    //this is the current location of the execution stack
    let context_func = new n_Function(func.name);
    let context = context_func.func_stack;
    let revs = new Array<Token>();
    for (let i = func.params.length - 1; i >= 0; i--) {
        let pr = g_stack.pop();
        let ty = func.params[i];
        if (pr) {
            if (pr.type === ty.type)
                revs.push(pr);
        } else
            throw func.name + " nothing to get from parent {" + g_func.name + "} stack";
    }
    revs.reverse().forEach(it => context.push(it)); //reverse and push for polish notation

    for (let i = 0; i < stack.length; i++) {
        let item = stack[i];
        if (item.type === TOKENS.FUNCTION) {
            switch (item.value) {
                case "print":
                    b_print(context);
                    break;
                case'+':
                    b_plus(context);
                    break;
                case'-':
                    b_minus(context);
                    break;
                case'*':
                    b_multiply(context);
                    break;
                case'/':
                    b_divide(context);
                    break;
                case'%':
                    b_mod(context);
                    break;
                case'swap':
                    b_swap(context);
                    break;
                case'drop':
                    b_drop(context);
                    break;
                case'copy':
                    b_copy(context);
                    break;
                case'max':
                    b_max(context);
                    break;
                case'min':
                    b_min(context);
                    break;
                case'ret':
                    b_ret(context, g_stack);
                    break;
            }
        } else if (item.type === TOKENS.IDENTIFIER) {
            let name2 = item.value as string;

            if (m_Stack.has(name2)) {
                let func_stack = m_Stack.get(name2);
                if (func_stack)
                    execute(func_stack, context_func);

            } else if (block.has(name2) || g_block.has(name2)) {
                if (block.has(name2))
                    context.push(block.get(name2)!);
                else
                    context.push(g_block.get(name2)!);

            }

        } else
            context.push(item);
    }
    /* //TODO this is gc :)
    if (fname != "_global")
        m_Block.delete(fname);
        */
}

// BUILTIN
function b_print(context: Stack) {
    const valid = [TOKENS.NUMBER, TOKENS.STRING, TOKENS.INTEGER, TOKENS.FLOAT, TOKENS.DOUBLE];
    let pr = context.pop();

    if (pr != undefined) {
        if (valid.includes(pr.type))
            console.log(pr.value);
        else
            throw pr.value + " is not a printable object";
    } else
        throw "no value to print";
}

function b_ret(f_context: Stack, g_context: Stack) {
    let last = f_context.pop();

    if (last != undefined) {
        g_context.push(last);
    } else
        throw "no value to return";
}

function b_copy(f_context: Stack) {
    let last = f_context.top();

    if (last != undefined) {
        f_context.push(last);
    } else
        throw "no value to copy";
}

function b_drop(f_context: Stack) {
    let last = f_context.pop();
    if (last == undefined)
        throw "no value to drop";
}

function b_swap(f_context: Stack) {
    let second = f_context.pop();
    let first = f_context.pop();
    if (first && second) {
        f_context.push(second, first);
    } else
        throw "no values to swap";
}

function b_plus(context: Stack) {
    const valid = [TOKENS.NUMBER, TOKENS.STRING, TOKENS.INTEGER, TOKENS.FLOAT, TOKENS.DOUBLE];
    let second = context.pop();
    let first = context.pop();
    if (first != undefined && second != undefined) {
        if (valid.includes(first.type) && valid.includes(second.type)) {
            if (typeof first.value == "string" && typeof second.value == "string")
                context.push({type: first.type, value: first.value + second.value});
            else if (typeof first.value == "number" && typeof second.value == "number")
                context.push({type: first.type, value: first.value + second.value});
            else
                throw first.value + " and " + second.value + " is not add-able";
        } else
            throw first.value + " and " + second.value + " is not add-able";
    } else
        throw "no value to add";
}

function b_minus(context: Stack) {
    const valid = [TOKENS.NUMBER, TOKENS.INTEGER, TOKENS.FLOAT, TOKENS.DOUBLE];
    let second = context.pop();
    let first = context.pop();
    if (first != undefined && second != undefined) {
        if (valid.includes(first.type) && valid.includes(second.type)) {
            if (typeof first.value == "number" && typeof second.value == "number")
                context.push({type: first.type, value: first.value - second.value});
        } else
            throw first.value + " and " + second.value + " is not subtractable";
    } else
        throw "no value to subtract";
}

function b_multiply(context: Stack) {
    const valid = [TOKENS.NUMBER, TOKENS.INTEGER, TOKENS.FLOAT, TOKENS.DOUBLE];
    let second = context.pop();
    let first = context.pop();
    if (first != undefined && second != undefined) {
        if (valid.includes(first.type) && valid.includes(second.type)) {
            if (typeof first.value == "number" && typeof second.value == "number")
                context.push({type: first.type, value: first.value * second.value});
        } else
            throw first.value + " and " + second.value + " is not multiply-able";
    } else
        throw "no value to multiply";
}

function b_divide(context: Stack) {
    const valid = [TOKENS.NUMBER, TOKENS.INTEGER, TOKENS.FLOAT, TOKENS.DOUBLE];
    let second = context.pop();
    let first = context.pop();
    if (first != undefined && second != undefined) {
        if (valid.includes(first.type) && valid.includes(second.type)) {
            if (typeof first.value == "number" && typeof second.value == "number")
                context.push({type: first.type, value: first.value / second.value});
        } else
            throw first.value + " and " + second.value + " is not divide-able";
    } else
        throw "no value to divide";
}

function b_mod(context: Stack) {
    const valid = [TOKENS.NUMBER, TOKENS.INTEGER, TOKENS.FLOAT, TOKENS.DOUBLE];
    let second = context.pop();
    let first = context.pop();
    if (first != undefined && second != undefined) {
        if (valid.includes(first.type) && valid.includes(second.type)) {
            if (typeof first.value == "number" && typeof second.value == "number")
                context.push({type: first.type, value: first.value % second.value});
        } else
            throw first.value + " and " + second.value + " is not mod-able";
    } else
        throw "no values to do a modulus operation";
}

function b_max(context: Stack) {
    const valid = [TOKENS.NUMBER, TOKENS.INTEGER, TOKENS.FLOAT, TOKENS.DOUBLE];
    let second = context.pop();
    let first = context.pop();
    if (first != undefined && second != undefined) {
        if (valid.includes(first.type) && valid.includes(second.type)) {
            if (typeof first.value == "number" && typeof second.value == "number")
                context.push({type: first.type, value: first.value > second.value ? first.value : second.value});
        } else
            throw first.value + " and " + second.value + " is not comparable for max";
    } else
        throw "no values to do a max comparison";
}

function b_min(context: Stack) {
    const valid = [TOKENS.NUMBER, TOKENS.INTEGER, TOKENS.FLOAT, TOKENS.DOUBLE];
    let second = context.pop();
    let first = context.pop();
    if (first != undefined && second != undefined) {
        if (valid.includes(first.type) && valid.includes(second.type)) {
            if (typeof first.value == "number" && typeof second.value == "number")
                context.push({type: first.type, value: first.value < second.value ? first.value : second.value});
        } else
            throw first.value + " and " + second.value + " is not comparable for min";
    } else
        throw "no values to do a min comparison";
}

//