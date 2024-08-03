const buffer = new Int8Array(new ArrayBuffer(255));
for(let i = 0; i < buffer.length; i++) {
    buffer[i] = Math.floor(Math.random() * 127);
}

Bun.write("lib", buffer);