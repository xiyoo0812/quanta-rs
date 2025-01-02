

print("hello world")

local x = 1
local y = 2

print(x, y, x+y)

local a = {a = 1, b =2} 
for k, v in pairs(a) do
	print(k, v)
end

local json = require "ljson"
local e = json.encode(a)

print("json encode:", e)

local d = json.decode(e)
for k, v in pairs(d) do
	print("json decode:", k, v)
end

function test()
	print("test func call")
end

function test1(a, b)
	print("test1 func call, args: ", a, b)
	return a, a + b, "ABC"
end