hostname = "wt-56909ab8";

% Open TCPStream and UDPSocket
tcp_socket = tcpclient(hostname, 8080);
udp_socket = udpport()

% Send subscription request on tcp
audio_sub_request = strcat("ecg subscribe ", string(udp_socket.LocalPort));
writeline(tcp_socket, audio_sub_request);


%gather packets until stop indicated
%ToDo make this loop work until the user exits
% loop currently only grabs 20 seconds of data

for i=1:1:20000
    % wait for atleast a sample
    while (udp_socket.NumBytesAvailable < 16)
        pause(0.1);
    end
    % UDP does not guarentee packet delivery or packet order

    % read a sample from udp stream
    sample.sys_time_us = read(udp_socket, 1, "uint64");
    sample.sample_index = read(udp_socket, 1, "uint64");
    sample.error = read(udp_socket, 1, "int32");
    sample.rtc_time_s = read(udp_socket, 1, "uint32");
    sample.ecg_reading = read(udp_socket, 1, "int32");
    sample.leads_off_reading_n = read(udp_socket, 1, "uint16");
    sample.leads_off_reading_p = read(udp_socket, 1, "uint16");
    sample

    % ToDo: Do something with sample

end

%convert data to ecg sample
audio_unsub_request = strcat("ecg unsubscribe ", string(udp_socket.LocalPort));
tcp_socket = tcpclient(hostname, 8080);
writeline(tcp_socket, audio_unsub_request);
tcp_socket = tcpclient(hostname, 8080);
% writeline(tcp_socket, "stop"); % this kills the daemon
