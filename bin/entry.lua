

print("hello world")

local x = 1
local y = 2

print(x, y, x+y)

local a = {a = 1, b =2} 
for k, v in pairs(a) do
	print(k, v)
end

set_path("LUA_PATH", "!/script/?.lua;;")

local json = require "ljson"
local e = json.encode(a)

print("json encode:", e)

local d = json.decode(e)
for k, v in pairs(d) do
	print("json decode:", k, v)
end

function test()
	print("test func call")
	require "fs"

	test_stdfs()
end

function test1(a, b)
	print("test1 func call, args: ", a, b)
	return a, a + b, "ABC"
end

local timer = require("ltimer")

local TIMER_ACCURYACY = 20

timer.insert(1000, 1000 / TIMER_ACCURYACY)

local last = timer.steady_ms()
function run()
	timer.sleep(20)
	local now_ms = timer.steady_ms()
	local esapped = now_ms - last
	local timerids =  timer.update(esapped // TIMER_ACCURYACY)
	for _, timerid in pairs(timerids) do
		print("timerid: ", timerid)
		timer.insert(timerid, 1000 / TIMER_ACCURYACY)
	end
	last = timer.steady_ms()
end
