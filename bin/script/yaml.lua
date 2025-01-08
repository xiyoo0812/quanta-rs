-- toml.lua

print("hello toml")

local cxml = [[
base: &base
  name: Everyone has same name
  id: 123456

foo: &foo
  <<: *base
  age: 10

bar: &bar
  <<: *base
  age: 20

]]


local yaml = require "lyaml"

local xlua = yaml.decode(cxml)
print("lyaml decode yaml:",  xlua)
local yxml = yaml.encode(xlua)
print("lyaml encode yaml:", yxml)

local ok = yaml.save("./bb.yaml", xlua)
print("lyaml save yaml:", ok)
local flua = yaml.open("./bb.yaml")
print("lyaml open yaml:", flua)
