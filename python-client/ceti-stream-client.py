import socket
import ctypes

pi_ip = '10.0.0.161'

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

# Open tcp command socket
cmd_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
cmd_socket.connect((pi_ip, 8080))
print("[ ] Listening on %s" % (cmd_socket.getsockname(),))

# subscribe to battery data
# Bug: space need after port #
cmd_socket.send("battery subscribe 12345 ".encode('utf-8'))


# Listen on UDP and 
sd = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sd.bind(('0.0.0.0', 12345))
print("[ ] Listening on %s" % (sd.getsockname(),))
while True:
    # Listen for datagram
    data = sd.recv(12345)

    # Parse C type into usable class
    bms_data = CetiBatterySample.from_buffer_copy(data)
    
    #print
    print(bms_data.sys_time_us)
