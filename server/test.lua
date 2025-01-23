--test.lua
--import("kernel.lua")

-- quanta.startup(function()
--     --初始化test
--     --[[
--     import("test/codec_test.lua")
--     import("test/json_test.lua")
--     import("test/pack_test.lua")
--     import("test/mongo_test.lua")
--     import("test/router_test.lua")
--     import("test/protobuf_test.lua")
--     import("test/http_test.lua")
--     import("test/rpc_test.lua")
--     import("test/log_test.lua")
--     import("test/timer_test.lua")
--     import("test/mysql_test.lua")
--     import("test/redis_test.lua")
--     import("test/stdfs_test.lua")
--     import("test/cmdline_test.lua")
--     import("test/ws_test.lua")
--     import("test/zipkin_test.lua")
--     import("test/clickhouse_test.lua")
--     import("test/url_test.lua")
--     import("test/udp_test.lua")
--     import("test/tcp_test.lua")
--     import("test/worker_test.lua")
--     import("test/lock_test.lua")
--     import("test/detour_test.lua")
--     import("test/bitset_test.lua")
--     import("test/lmdb_test.lua")
--     import("test/unqlite_test.lua")
--     import("test/sqlite_test.lua")
--     import("test/ssl_test.lua")
--     import("test/xml_test.lua")
--     import("test/yaml_test.lua")
--     import("test/toml_test.lua")
--     import("test/csv_test.lua")
--     import("test/smdb_test.lua")
--     import("test/profile_test.lua")
--     import("test/pgsql_test.lua")
--     ]]
--     import("test/stdfs_test.lua")
-- end)

function test()
    local x1 = wrapper()
    local x2 = wrapper1(1)
    local x3 = wrapper2(1, 2)
    print("test", x1, x2, x3)
end

function test1(a, b)
    print("test1", a, b)
    return 1, 2, "test1"
end

global = {
    test = function(a)
        print("global: test: ", a)
    end
}

function test2()
    local tt = luakit.test()
    tt.test = function(a)
        print("object call: test: ", a)
    end
    local x1 = tt.some_method1(12)
    local x2 = tt.some_method2(12)
    local x3 = tt.some_method3(12)
    local x4 = tt.some_method4(12)
    print(x1, x2, x3, x4)

    local tt2 = luakit.test2()
    print("test i1", tt2.i)
    local t1 = tt2.some_method_1(11)
    local t2 = tt2.some_method_2(11)
    local t3 = tt2.some_method_3(11)
    local t4 = tt2.some_method_4(11)
    print("test i2", tt2.i)
    print(t1, t2, t3, t4)
    tt2.i = 100
    print("test i3", tt2.i)

    collectgarbage("collect")
end
--import("test/stdfs_test.lua")
