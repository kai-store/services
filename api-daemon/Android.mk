#
# Glue to call the cargo based build system.
#

LOCAL_PATH:= $(call my-dir)
GONK_DIR := $(abspath $(LOCAL_PATH)/../../)
DAEMON_ROOT := $(GONK_DIR)/services/api-daemon

# Add the api-daemon executable.
include $(CLEAR_VARS)

API_DAEMON_EXEC := prebuilts/armv7-linux-androideabi/api-daemon

LOCAL_MODULE := api-daemon
LOCAL_MODULE_CLASS := EXECUTABLES
LOCAL_MODULE_TAGS := optional
LOCAL_SHARED_LIBRARIES := libc libm libdl liblog libssl libcutils
LOCAL_SRC_FILES := $(API_DAEMON_EXEC)
LOCAL_MODULE_PATH := $(TARGET_OUT)/kaios
include $(BUILD_PREBUILT)

$(LOCAL_BUILT_MODULE):
	@echo "api-daemon: $(API_DAEMON_EXEC)"

$(LOCAL_INSTALLED_MODULE):
	@mkdir -p $(@D)
	@cp $(DAEMON_ROOT)/daemon/config-device.toml $(TARGET_OUT)/kaios/config.toml
	@cp -R $(DAEMON_ROOT)/prebuilts/http_root $(TARGET_OUT)/kaios/
	@cp $(DAEMON_ROOT)/$(API_DAEMON_EXEC) $(TARGET_OUT)/kaios/
