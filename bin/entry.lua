-- entry.lua


function startup()
	set_path("LUA_PATH", "!/script/?.lua;;")

	require "fs"
	require "json"
	require "timer"
	require "toml"
	require "yaml"
	require "test"
end

function run()
	timer_update()
end
