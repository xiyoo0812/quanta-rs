
~
robot.protoncmd_cs*^
BtNodeId
BT_ZERO 
BT_LOGIN�
BT_LOGIN_UNIT�
	BT_LOGOUT�
BT_LOGIN_WAIT�bproto3
�
ncmd_cs.protoncmd_cs*�
NCmdId
NID_QUANTA_ZERO 
NID_HEARTBEAT_REQ�
NID_HEARTBEAT_RES� 
NID_LOGIN_ACCOUNT_LOGIN_REQ�N 
NID_LOGIN_ACCOUNT_LOGIN_RES�N!
NID_LOGIN_ACCOUNT_RELOAD_REQ�N!
NID_LOGIN_ACCOUNT_RELOAD_RES�N
NID_LOGIN_RANDOM_NAME_REQ�N
NID_LOGIN_RANDOM_NAME_RES�N
NID_LOGIN_ROLE_CREATE_REQ�N
NID_LOGIN_ROLE_CREATE_RES�N
NID_LOGIN_ROLE_CHOOSE_REQ�N
NID_LOGIN_ROLE_CHOOSE_RES�N
NID_LOGIN_ROLE_DELETE_REQ�N
NID_LOGIN_ROLE_DELETE_RES�N
NID_LOGIN_ROLE_LOGIN_REQ�N
NID_LOGIN_ROLE_LOGIN_RES�N
NID_LOGIN_ROLE_LOGOUT_REQ�N
NID_LOGIN_ROLE_LOGOUT_RES�N
NID_LOGIN_ROLE_RELOAD_REQ�N
NID_LOGIN_ROLE_RELOAD_RES�N
NID_LOGIN_ROLE_KICKOUT_NTF�N
NID_ENTITY_ENTER_SCENE_NTF�^
NID_ENTITY_LEAVE_SCENE_NTF�^
NID_UTILITY_GM_COMMAND_REQ�U
NID_UTILITY_GM_COMMAND_RES�Ubproto3
�
login.protoncmd_cs"f
	role_info
role_id (RroleId
name (	Rname
gender (Rgender
model (Rmodel"�
login_account_login_req
openid (	Ropenid
session (	Rsession2
platform (2.ncmd_cs.platform_typeRplatform
	device_id	 (	RdeviceId"{
login_account_login_res

error_code (R	errorCode
user_id (RuserId(
roles (2.ncmd_cs.role_infoRroles"i
login_account_reload_req
openid (	Ropenid
session (	Rsession
	device_id (	RdeviceId"|
login_account_reload_res

error_code (R	errorCode
user_id (RuserId(
roles (2.ncmd_cs.role_infoRroles"
login_random_name_req"J
login_random_name_res

error_code (R	errorCode
name (	Rname"K
	rolemodel
model (Rmodel
color (Rcolor
head (Rhead"t
login_role_create_req
user_id (RuserId
name (	Rname
gender (Rgender
custom (Rcustom"^
login_role_create_res

error_code (R	errorCode&
role (2.ncmd_cs.role_infoRrole"I
login_role_choose_req
user_id (RuserId
role_id (RroleId"�
login_role_choose_res

error_code (R	errorCode
addrs (	Raddrs
port (Rport
lobby (Rlobby
token (Rtoken
role_id (RroleId"I
login_role_delete_req
user_id (RuserId
role_id (RroleId"6
login_role_delete_res

error_code (R	errorCode"t
login_role_login_req
open_id (	RopenId
role_id (RroleId
lobby (Rlobby
token (Rtoken"K
login_role_login_res

error_code (R	errorCode
token (Rtoken"0
login_role_logout_req
role_id (RroleId"6
login_role_logout_res

error_code (R	errorCode"u
login_role_reload_req
open_id (	RopenId
role_id (RroleId
lobby (Rlobby
token (Rtoken"L
login_role_reload_res

error_code (R	errorCode
token (Rtoken"0
login_role_kickout_ntf
reason (Rreason"�
entity_enter_scene_ntf
id (Rid
scene_id (RsceneId
type (Rtype
map_id (RmapId
pos_x (RposX
pos_y (RposY
pos_z (RposZ
dir_y (RdirY"C
entity_leave_scene_ntf
id (Rid
scene_id (RsceneId*:
platform_type
PLATFORM_GUEST 
PLATFORM_PASSWORDbproto3
�
errorcode.protoncmd_cs*�
	ErrorCode
FRAME_SUCCESS 
FRAME_FAILED
FRAME_TOOFAST
FRAME_PARAMS
FRAME_UPHOLD
LOGIN_PLATFORM_ERROR�
LOGIN_VERIFY_FAILED�
LOGIN_SERVER_UPHOLD�
LOGIN_ACCOUTN_BANS�
LOGIN_ACCOUTN_INLINE�
LOGIN_ACCOUTN_OFFLINE�
LOGIN_ROLE_NOT_EXIST�
LOGIN_ROLE_NUM_LIMIT�
LOGIN_ROLE_NAME_EXIST�
LOGIN_ROLE_IS_INLINE�
LOGIN_ROLE_IS_OFFLINE�
LOGIN_ROLE_TOKEN_ERR�
KICK_DEVICE_REPLACE�
KICK_SERVER_UPHOLD�
KICK_ACCOUTN_BANS�bproto3
�
common.protoncmd_cs";
heartbeat_req
serial (Rserial
time (Rtime"Z
heartbeat_res
serial (Rserial
time (Rtime

error_code (R	errorCode"2
utility_gm_command_req
command (	Rcommand"T
utility_gm_command_res

error_code (R	errorCode
	error_msg (	RerrorMsgbproto3