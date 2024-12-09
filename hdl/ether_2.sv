`default_nettype none


module ether_2(
    input wire clk,
    input wire rst,
    input wire [1:0] ether_rxd,
    input wire ether_crsdv,
    output logic [12000:0] m00_axis_tdata,
    output logic m00_axis_tvalid
);


enum logic [2:0] {IDLE,SFD,DESTINATION,SOURCE,LENGTH,DATA_STREAM,CRC  } state;

localparam SFD_CODE = 8'b11010101;
localparam FRC_CODE = 32'h38FB2284;
localparam MAC_ADDRESS = 48'hBEEFDEADFEFE;
logic [7:0] curr_byte;
logic [47:0] source_address;
logic [47:0] destination_address;
logic [15:0] data_length;
logic [12000:0] data;
logic data_uncorrupted;
logic end_of_frame;

logic [4:0] bit_count;
logic [11:0] byte_count;
logic [5:0] preamble_count;
logic [7:0] test;

assign test= {ether_rxd,curr_byte[7:2]};



logic crc_input_valid;

logic [31:0] frame_check_sequence;


crc32 checksum
            (   .clk(clk),
                .rst(rst || (~crc_input_valid && ~end_of_frame)),
                .axiiv(crc_input_valid),
                .axiid(ether_rxd),
                .axiod(frame_check_sequence)
            );



initial begin
    crc_input_valid = 0;
    state           = IDLE;
    byte_count      = 0;
    bit_count       = 0;
    preamble_count  = 0;
    curr_byte       = 0;
    data            = 0;
    end_of_frame    = 0;
    source_address  = 0;
    destination_address = 0;
end
// 38FB2284
assign data_uncorrupted = (frame_check_sequence == FRC_CODE);
assign m00_axis_tvalid = (end_of_frame && data_uncorrupted && destination_address == MAC_ADDRESS);
assign m00_axis_tdata = data;



always_ff @( posedge clk ) begin : stateMachine
    if (rst)begin
        
        state <= IDLE;
        byte_count<=0;
        bit_count<=0;
        preamble_count<=0;
        data<=0;
        curr_byte<=0;
        end_of_frame<=0;
        

    end else begin

        case (state)
            IDLE: begin
                if(ether_crsdv == 1)begin
                if(ether_rxd == 2'b01)begin
                    if(preamble_count== 5'd27)begin
                        state<=SFD;
                        preamble_count<=0;
                    end else begin
                        preamble_count<=preamble_count+1;
                    end
                end else begin
                    preamble_count<=0;
                end
                end
                if(end_of_frame)begin
                    end_of_frame<=0;
                end
                
            end
            SFD:begin
                curr_byte<={ether_rxd,curr_byte[7:2]};
                if(bit_count == 6)begin
                    if({ether_rxd,curr_byte[7:2]} == SFD_CODE)begin
                        state<= DESTINATION;
                        crc_input_valid<=1;
                    end else begin
                        state <= IDLE;
                    end
                    bit_count<=0;
                end else begin
                    bit_count<=bit_count+2;
                end
            end
            DESTINATION:begin
                curr_byte<={ether_rxd,curr_byte[7:2]};
                if(bit_count == 6)begin
                    if(byte_count == 5)begin
                        destination_address<={data[39:0],ether_rxd,curr_byte[7:2]};
                        state<=SOURCE;
                        byte_count<=0;
                    end else begin
                        byte_count<=byte_count+1;
                        data<={data[11992:0],ether_rxd,curr_byte[7:2]};
                        
                    end
                    bit_count<=0;
                end else begin
                    bit_count<=bit_count+2;
                end
            end
            SOURCE:begin
                curr_byte<={ether_rxd,curr_byte[7:2]};
                if(bit_count == 6)begin
                    if(byte_count == 5)begin
                        source_address<={data[39:0],ether_rxd,curr_byte[7:2]};
                        state<=LENGTH;
                        byte_count<=0;
                    end else begin
                        byte_count<=byte_count+1;
                        data<={data[11992:0],ether_rxd,curr_byte[7:2]};  
                    end
                    bit_count<=0;
                end else begin
                    bit_count<=bit_count+2;
                end
            end
            LENGTH:begin
                curr_byte<={ether_rxd,curr_byte[7:2]};
                if(bit_count == 6)begin
                    if(byte_count == 1)begin
                        data_length<={data[7:0],ether_rxd,curr_byte[7:2]};
                        state<=DATA_STREAM;
                        data<=0;
                        byte_count<=0;
                    end else begin
                        byte_count<=byte_count+1;
                        data<={data[11992:0],ether_rxd,curr_byte[7:2]};
                        
                    end
                    bit_count<=0;
                end else begin
                    bit_count<=bit_count+2;
                end
            end
            DATA_STREAM:begin
                curr_byte<={ether_rxd,curr_byte[7:2]};
                if(bit_count == 6)begin
                    if(byte_count == data_length-1)begin
                        data<={data[11992:0],ether_rxd,curr_byte[7:2]};
                        state<=CRC;
                        byte_count<=0;
                    end else begin
                        byte_count<=byte_count+1;
                        data<={data[11992:0],ether_rxd,curr_byte[7:2]};
                    end
                    bit_count<=0;
                end else begin
                    bit_count<=bit_count+2;
                end
            end
            CRC:begin
                if(bit_count == 6)begin
                    if(byte_count == 3)begin
                        state<=IDLE;
                        end_of_frame<=1;
                        crc_input_valid<=0;
                        byte_count<=0;
                    end else begin
                        byte_count<=byte_count+1;
                    end
                    bit_count<=0;
                end else begin
                    bit_count<=bit_count+2;
                    
                end
            end
  
        endcase



    end
    
end


endmodule


`default_nettype wire