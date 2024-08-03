let code = "";

for(let i = 1; i <= 10000; ++i) {
    if(i % 3 === 0 && i % 5 === 0) {
        code += "\nfizzBuzzString,"
    } else if(i % 3 === 0) {
        code += "\nfizzString,"
    } else if(i % 5 === 0) {
        code += "\nbuzzString,"
    } else {
        code += `"${i}",`;
    }
}

console.log(code);

// let fizzBuzz = length => Array.from({length}, (_, i) => (i++, `${i % 3 === 0 ? "Fizz" : ""}${i % 5 === 0 ? "Buzz" : ""}` || `${i}`));