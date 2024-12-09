from scapy import *
from scapy.layers.l2 import Ether
from scapy.packet import Raw
from scapy.sendrecv import sendp
from time import sleep





#!/usr/bin/env python3


# network_packet="".join([normal_to_network(byte) for byte in packet])
# print(hex(int("".join(packet),2)))
# print(hex(int(network_packet,2)))
# fcs=crc32_calculator(int(network_packet,2))
# print(hex(int(fcs,2)))
# if len(fcs)%8!=0:
#     fcs="0"*(8-(len(fcs))%8)+fcs
# fcs=[fcs[8*i:8*i+8] for i in range(len(fcs)//8)]

# fcs=[i[::-1] for i in fcs]
# network_packet="".join([normal_to_dibit_network(byte) for byte in preamble+sfd+packet+fcs])
# # print(hex(int("".join([normal_to_dibit_network(byte) for byte in packet]),2)))
# di_bits=[network_packet[4*i:4*i+4] for i in range(len(network_packet)//4)]
# # print(di_bits)
# tester.input_driver.append({ "type": "burst", "contents": { "data": [int(i,2) for i in di_bits] } })


def normal_to_network(byte):
    return byte[::-1]

def normal_to_dibit_network(byte):
    split_byte=[byte[4*i:4*i+4] for i in range(len(byte)//4)]
    split_byte.reverse()

    return "".join(split_byte)

def fields_to_packet(fields):
    return "".join(fields)

def crc32_calculator(data):
    a = data.to_bytes((data.bit_length() + 7) // 8,byteorder='big')
    crc = 0xffffffff
    # print(len(a))
    for x in a:
        crc ^= x << 24;
        for k in range(8):
            crc = (crc << 1) ^ 0x04c11db7 if crc & 0x80000000 else crc << 1
        # print(x)
    crc = ~crc
    crc &= 0xffffffff
    return bin(crc)[2:]

def integer_to_packet_data(integer):
    binary=byte_pad(bin(integer)[2:])

    length=byte_pad(bin(len(binary)//8)[2:])
    if len(length)<16:
        length="0"*(16-len(length))+length
    # print(length)
    
    
    data=[binary[8*i:8*i+8] for i in range(len(binary)//8)]
    length=[length[8*i:8*i+8] for i in range(len(length)//8)]
    # print([\
    #     normal_to_dibit_network(i) for i in length+data])
    return length+data
    

def byte_pad(s):
    s_padded=s
    if len(s)%8!=0:
        s_padded="0"*(8-(len(s))%8)+s
    return s_padded

preamble=["01010101"]*7
sfd=["11010101"]
source_address=byte_pad(bin(int("000000000000",16))[2:])
destination_address=byte_pad(bin(int("BEEFDEADFEFE",16))[2:])
source_address=[source_address[8*i:8*i+8] for i in range(len(source_address)//8)]
destination_address=[destination_address[8*i:8*i+8] for i in range(len(destination_address)//8)]




if __name__ == "__main__":
    # for i in ("00","01","10","11"):
        print("hello")
        # print(heyhgvcx("\x81"*46))
        print("\91")
        frame=Ether(src="00:00:00:00:00:00", dst="be:ef:de:ad:ab:cd", type=48)/Raw(b"\xEB\xFE"*24)
        # packet=source_address+destination_address+integer_to_packet_data(int("21")*46)
        # network_packet="".join([normal_to_network(byte) for byte in packet])
        # fcs=crc32_calculator(int(network_packet,2))
        # if len(fcs)%8!=0:
        #     fcs="0"*(8-(len(fcs))%8)+fcs
        # fcs=[fcs[8*i:8*i+8] for i in range(len(fcs)//8)]
        
        # fcs=[i[::-1] for i in fcs]
        # fcs=hex(int("".join(fcs),2))[2:]
        # fcs=[r"\x"+fcs[2*i:2*i+2] for i in range(len(fcs)//2)]
        # print("".join(fcs))
        # frame=frame/Raw("".join(fcs))
        for i in range(50):
            sleep(3)
            sendp(frame, iface="en12")
        # sendp(frame, iface="en12")
        # sendp(frame, iface="en12")
        # sendp(frame, iface="en12")
        # sendp(frame, iface="en12")
        # sendp(frame, iface="en12")
        # sendp(frame, iface="en12")
        # sendp(frame, iface="en12")
        # sendp(frame, iface="en12")
        # sendp(frame, iface="en12")
        # sendp(frame, iface="en12")
        # sendp(frame, iface="en12")
        # sendp(frame, iface="en12")
        # sendp(frame, iface="en12")
        # sendp(frame, iface="en12")
        # sendp(frame, iface="en12")
        

        
        
        
        # sleep(1)
        frame=Ether(src="00:00:00:00:00:00", dst="be:ef:de:ad:fe:fe", type=46)/Raw(b"\xCA"*46)
        # packet=source_address+destination_address+integer_to_packet_data(int("\x"+i)*46)
        # network_packet="".join([normal_to_network(byte) for byte in packet])
        # fcs=crc32_calculator(int(network_packet,2))
        # if len(fcs)%8!=0:
        #     fcs="0"*(8-(len(fcs))%8)+fcs
        # fcs=[fcs[8*i:8*i+8] for i in range(len(fcs)//8)]
        
        # fcs=[i[::-1] for i in fcs]
        # frame=frame/Raw(hex(int("".join(fcs),2))[2:])
        # sendp(frame, iface="en12")
        # sleep(1)
    # normal_to_network("1223485859669")