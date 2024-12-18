local LibDeflate = require("LibDeflate")
require("base64")
print(to_base64(LibDeflate:CompressDeflate(io.read("*a"), {level = 9})))