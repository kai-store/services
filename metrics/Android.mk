#
# Glue to call the cargo based build system.
#

LOCAL_PATH := $(call my-dir)
GONK_DIR   := $(abspath $(LOCAL_PATH)/../../)

# Add the metrics_daemon executable.
include $(CLEAR_VARS)

METRICS_ROOT := $(GONK_DIR)/services/metrics
METRICS_EXEC := prebuilts/armv7-linux-androideabi/metrics_daemon

LOCAL_MODULE       := metrics_daemon
LOCAL_MODULE_CLASS := DATA
LOCAL_MODULE_TAGS  := optional
LOCAL_SRC_FILES    := $(METRICS_EXEC)
LOCAL_MODULE_PATH  := $(TARGET_OUT)/kaios
LOCAL_SHARED_LIBRARIES := libc libm libdl liblog
include $(BUILD_PREBUILT)

$(LOCAL_BUILT_MODULE): config.json
	@echo "metrics_daemon: $(METRICS_EXEC)"

$(LOCAL_INSTALLED_MODULE):
	@mkdir -p $(@D)
	@cp $(METRICS_ROOT)/$(METRICS_EXEC) $(@D)

# Add the config file
include $(CLEAR_VARS)
LOCAL_MODULE       := config.json
LOCAL_MODULE_CLASS := DATA
LOCAL_MODULE_TAGS  := optional
LOCAL_SRC_FILES    := config.json
LOCAL_MODULE_PATH  := $(TARGET_OUT)/kaios
include $(BUILD_PREBUILT)
