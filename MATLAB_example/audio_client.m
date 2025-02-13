hostname = "wt-56909ab8";

% Open TCPStream and UDPSocket
tcp_socket = tcpclient(hostname, 8080);
udp_socket = udpport()

% Send subscription request on tcp
audio_sub_request = strcat("audio subscribe ", string(udp_socket.LocalPort));
writeline(tcp_socket, audio_sub_request);
while (udp_socket.NumBytesAvailable < 32)
    pause(0.1);
end

%gather packets until stop indicated
% for i=1:1:1
    % UDP does not guarentee packet delivery or packet order
    % TODO: packet index is to rebuild order here
    packet_index = read(udp_socket, 1, "uint32");

    % 16-bit, 96 kSPS, 3 channel interleaved big endian
    raw_data = uint8(read(udp_socket, 1494, "uint8"));
    % TODO: Process data/rebuild samples
% en

%convert data to ecg sample
audio_unsub_request = strcat("audio unsubscribe ", string(udp_socket.LocalPort));
tcp_socket = tcpclient(hostname, 8080);
writeline(tcp_socket, audio_unsub_request);
tcp_socket = tcpclient(hostname, 8080);
writeline(tcp_socket, "stop");
