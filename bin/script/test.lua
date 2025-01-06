-- test.lua

print("hello test")

function test()
	print("test func call")
    local w1 = wrapper()
    local w2 = wrapper1(2)
    local w3 = wrapper2(1, 2)
	print("test func call wrapper, rets: ", w1, w2, w3)
end

function test1(a, b)
	print("test1 func call, args: ", a, b)
	return a, a + b, "ABC"
end
