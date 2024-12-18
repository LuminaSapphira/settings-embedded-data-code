local LibDeflate = require("LibDeflate")
require("base64")
local luacode_comp = settings.startup["settings-embedded-data-code-data-stage-code"].value
if string.len(luacode_comp) == 0 then
    return
end

local decomp, err = LibDeflate:DecompressDeflate(from_base64(luacode_comp))
if decomp then
    log("Loading lua from settings, I hope you know what you're doing : [\n" .. decomp .. "\n]")
else
    error("Error occured: " .. err)
end
load(decomp)()
