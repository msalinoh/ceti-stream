#import 

# c_type definition
class CetiBatterySample(ctypes.Structure):
    _fields_ = (
        ('sys_time_us', ctypes.c_int64),
        ('error', ctypes.c_int32),
        ('rtc_time_s', ctypes.c_int),
        ('cell_voltage_v', ctypes.c_double * 2),
        ('cell_temperature_c', ctypes.c_double * 2),
        ('current_ma', ctypes.c_double),
        ('state_of_charge', ctypes.c_double),
        ('status', ctypes.c_uint16),
        ('protection_alert', ctypes.c_uint16),
    )

class CetiAudioDatagram(ctypes.Structure):
    _fields_ = (
        ('packet_index', ctypes.c_uint32),
        ('raw_data', ctypes.c_uint8*1494), # this is big endian/needs to be reformatted
        #('audio_data', ((ctypes.c_uint8*2)*3)*249), 
    )

class CetiImuQuatSample(ctypes.Structure):
    _fields_ = (
        ('sys_time_us', ctypes.c_int64),
        ('reading_delay_us', ctypes.c_int64),
        ('rtc_time_s', ctypes.c_int),
        ('i', ctypes.c_int16_t),
        ('j', ctypes.c_int16_t),
        ('k', ctypes.c_int16_t),
        ('real', ctypes.c_int16_t),
        ('accuracy', ctypes.c_int16_t),
    )

class CetiImuAccelSample(ctypes.Structure):
    _fields_ = (
        ('sys_time_us', ctypes.c_int64),
        ('reading_delay_us', ctypes.c_int64),
        ('rtc_time_s', ctypes.c_int),
        ('x', ctypes.c_int16_t),
        ('y', ctypes.c_int16_t),
        ('z', ctypes.c_int16_t),
        ('accuracy', ctypes.c_int16_t),
    )

class CetiImuGyroSample(ctypes.Structure):
    _fields_ = (
        ('sys_time_us', ctypes.c_int64),
        ('reading_delay_us', ctypes.c_int64),
        ('rtc_time_s', ctypes.c_int),
        ('x', ctypes.c_int16_t),
        ('y', ctypes.c_int16_t),
        ('z', ctypes.c_int16_t),
        ('accuracy', ctypes.c_int16_t),
    )

class CetiImuMagSample(ctypes.Structure):
    _fields_ = (
        ('sys_time_us', ctypes.c_int64),
        ('reading_delay_us', ctypes.c_int64),
        ('rtc_time_s', ctypes.c_int),
        ('x', ctypes.c_int16_t),
        ('y', ctypes.c_int16_t),
        ('z', ctypes.c_int16_t),
        ('accuracy', ctypes.c_int16_t),
    )

class CetiLightSample(ctypes.Structure):
    _fields_ = (
        ('sys_time_us', ctypes.c_int64),
        ('rtc_time_s', ctypes.c_int),
        ('error', ctypes.c_int32),
        ('visible', ctypes.c_int),
        ('infrared', ctypes.c_int),
    )

class CetiPressureSample(ctypes.Structure):
    _fields_ = (
        ('sys_time_us', ctypes.c_int64),
        ('rtc_time_s', ctypes.c_int),
        ('error', ctypes.c_int32),
        ('pressure_bar', ctypes.c_double),
        ('temperature_c', ctypes.c_double),
    )

class CetiRecoverySample(ctypes.Structure):
    _fields_ = (
        ('sys_time_us', ctypes.c_int64),
        ('rtc_time_s', ctypes.c_int),
        ('nmea_sentence', ctypes.c_char*96),
    )