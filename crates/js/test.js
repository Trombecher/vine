const firstDigitChars = "_$abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const subsequentDigitChars = "_$abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/**
 * @param {number} n
 * @returns {string}
 */
function encode(n) {
    /**
     * @type {string}
     */
    let s = firstDigitChars[n % firstDigitChars.length];
    n = Math.floor(n / firstDigitChars.length);

    while(n > 0) {
        s += subsequentDigitChars[n % (subsequentDigitChars.length + 1) - 1];
        n = Math.floor(n / (subsequentDigitChars.length + 1));
    }

    return s.split("").reverse().join("");
}

/*
for(let i = 0; i < 1000; i++) {
    console.log(`${i} => ${encode(i)}`);
}
*/

/**
 * @param {string} chars
 * @returns {string}
 */
function generateReverseLookup(chars) {
    let array = new Array(256).fill(0);

    for(let i = 0; i < chars.length; i++)
        array[chars.charCodeAt(i)] = i;

    return array.map(i => `${i}`.padStart(3, "0")).join(", ");
}

console.log(generateReverseLookup(subsequentDigitChars))