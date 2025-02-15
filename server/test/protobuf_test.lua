--protobuf_test.lua

local protobuf_mgr  = quanta.get("protobuf_mgr")

local log_debug     = logger.debug
local NCmdId        = ncmd_cs.NCmdId

local pb_data  = {
    serial = 1,
    time = 801000000
}
local pb_str1 = protobuf_mgr:encode("NID_HEARTBEAT_REQ", pb_data)
local data1 = protobuf_mgr:decode("NID_HEARTBEAT_REQ", pb_str1)
local pb_str2 = protobuf_mgr:encode(NCmdId.NID_HEARTBEAT_REQ, pb_data)
local data2 = protobuf_mgr:decode(NCmdId.NID_HEARTBEAT_REQ, pb_str2)

log_debug("protobuf encode name:{}, {}", #pb_str1, type(pb_str1))
log_debug("protobuf encode enum:{}, {}", #pb_str2, type(pb_str2))
for k, v in pairs(data1) do
    log_debug("protobuf decode name:{}", v)
end
for k, v in pairs(data2) do
    log_debug("protobuf decode enum:{}", v)
end

local ppb_data = {error_code=1001014162,role={role_id=107216333761938434,name="aaa", gender = 2, model = 3}}
local ppb_str = protobuf_mgr:encode_byname("ncmd_cs.login_role_create_res", ppb_data)
local pdata = protobuf_mgr:decode_byname("ncmd_cs.login_role_create_res", ppb_str)

log_debug("protobuf encode:{}, {}", #ppb_str, type(ppb_str))
for k, v in pairs(pdata) do
    if type(v) == "table" then
        for kk, vv in pairs(v) do
            log_debug("protobuf decode {}.{}={}", k, kk, vv)
        end
    else
        log_debug("protobuf decode {}={}", k, v)
    end
end
