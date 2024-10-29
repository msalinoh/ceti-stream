import socket
import ctypes

#ToDo: set to pi's lan ip
pi_ip = '10.0.0.161'

# Open tcp command socket
cmd_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
cmd_socket.connect((pi_ip, 8080))
print("[ ] Listening on %s" % (cmd_socket.getsockname(),))

# subscribe to battery data
# Bug: space need after port #
cmd_socket.send("audio subscribe 12340 ".encode('utf-8'))


# Listen on UDP and 
sd = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sd.bind(('0.0.0.0', 12340))
print("[ ] Listening on %s" % (sd.getsockname(),))
while True:
    # Listen for datagram
    data = sd.recv(12345)
    packet_index = c_uint32.from_buffer_copy(data[:4]) # note: the index will wrap once in
    print("Received packet #: %d" % packet_index)
    #data is 3 channel interleaved, 16-bit big endian
    raw_audio_bytes = data[4:] 

