-- json.lua

print("hello json")

function test_json()
    local json = require "ljson"

    local a = {a = 1, b =2}
    local e = json.encode(a)

    print("json encode:", e)

    local d = json.decode(e)
    for k, v in pairs(d) do
        print("json decode:", k, v)
    end
end

test_json()