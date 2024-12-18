local LibDeflate = require("LibDeflate")
require("base64")
local x = LibDeflate:DecompressDeflate(from_base64(io.read("*a")))
print(x)
