

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
	local stdfs = require "lstdfs"
	local dirs = stdfs.dir("./")
	print("test func call")
	for i, info in ipairs(dirs) do
		print(string.format("dir %d: {name = %s, type = %s} ", i, info.name, info.type))
	end
	local fstem = stdfs.stem("./bb/xx.lua")
	print ("file stem:", fstem)
	local filename = stdfs.filename("./bb/xx.lua")
	print ("file name:", filename)
	local ftype = stdfs.filetype("./aa.lua")
	print ("file type:", ftype)
	local fsize = stdfs.file_size("./aa.lua")
	print ("file size:", fsize)
	local isdir = stdfs.is_directory("./bb")
	print ("is dir:", isdir)
	local ok, err = stdfs.mkdir("./bb/cc/dd")
	print ("mkdir:", ok, err)
end

function test1(a, b)
	print("test1 func call, args: ", a, b)
	return a, a + b, "ABC"
end
