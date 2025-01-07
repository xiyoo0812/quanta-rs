-- timer.lua

print("hello timer")

local TIMER_ACCURYACY = 20
local timer = require("ltimer")

timer.insert(1000, 1000 / TIMER_ACCURYACY)

local last = timer.steady_ms()
function timer_update()
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