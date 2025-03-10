﻿<?xml version="1.0" encoding="utf-8"?>
<Project ToolsVersion="4.0" xmlns="http://schemas.microsoft.com/developer/msbuild/2003">
  {{% local TEMPS, GROUPS = {}, {} %}}
  {{% local ARGS = {RECURSION = RECURSION, OBJS = OBJS, EXCLUDE_FILE = EXCLUDE_FILE } %}}
  {{% local CINCLUDES, CSOURCES = COLLECT_SOURCES(WORK_DIR, SRC_DIRS, ARGS) %}}
  <ItemGroup>
  {{% for _, CINC in pairs(CINCLUDES or {}) do %}}
    <ClInclude Include="{{%= CINC[1] %}}">
      {{% TEMPS[CINC[2]] = true %}}
      <Filter>{{%= CINC[2] %}}</Filter>
    </ClInclude>
  {{% end %}}
  </ItemGroup>
  <ItemGroup>
  {{% for _, CSRC in pairs(CSOURCES or {}) do %}}
    <ClCompile Include="{{%= CSRC[1] %}}">
      {{% local FGROUP = CSRC[2] %}}
      {{% TEMPS[FGROUP] = true %}}
      {{% local i, j = FGROUP:find("\\") %}}
      {{% while i do %}}
        {{% local TITLE = FGROUP:sub(1, i - 1) %}}
        {{% TEMPS[TITLE] = true %}}
        {{% i, j = FGROUP:find("\\", j + 1) %}}
      {{% end %}}
      <Filter>{{%= CSRC[2] %}}</Filter>
    </ClCompile>
  {{% end %}}
  </ItemGroup>
  <ItemGroup>
  {{% for GROUP in pairs(TEMPS or {}) do %}}
    {{% table.insert(GROUPS, GROUP) %}}
  {{% end %}}
  {{% table.sort(GROUPS, function(a, b) return a < b end) %}}
  {{% for _, GROUP in pairs(GROUPS or {}) do %}}
    <Filter Include="{{%= GROUP %}}">
      <UniqueIdentifier>{{{%= GUID_NEW(GROUP) %}}}</UniqueIdentifier>
    </Filter>
  {{% end %}}
  </ItemGroup>
</Project>