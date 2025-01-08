-- toml.lua

print("hello toml")

local ctoml = [[
[animals]
cats = [ "tiger", "lion", "puma" ]
birds = [ "macaw", "pigeon", "canary" ]
fish = [ "salmon", "trout", "carp" ]
]]

local toml = require "ltoml"

local xlua = toml.decode(ctoml)
print("ltoml decode toml:",  xlua)

local txml = toml.encode(xlua)
print("ltoml encode toml:", txml)

local ok = toml.save("bb.toml", xlua)
print("ltoml save toml:", ok)

local flua = toml.open("./bb.toml")
print("ltoml open toml:", flua)
