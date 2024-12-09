`default_nettype none


module ether_4(
    input wire clk,
    input wire rx_clk,
    input wire rst,
    input wire [3:0] ether_rxd,
    input wire ether_crsdv,
    output logic [2400:0] m00_axis_tdata,
    output logic m00_axis_tvalid,
    output logic [31:0] debug
);


enum logic [3:0] {IDLE,SFD,DESTINATION,SOURCE,LENGTH,DATA_STREAM,CRC,FAULTY  } state;

localparam SFD_CODE = 8'b11010101;
localparam FRC_CODE = 32'h38FB2284;
localparam MAC_ADDRESS = 48'hBEEFDEADABCD;
logic [7:0] curr_byte;
logic [47:0] source_address;
logic [47:0] destination_address;
logic [15:0] data_length;
logic [2400:0] data;
logic data_uncorrupted;
logic end_of_frame;

logic [4:0] bit_count;
logic [11:0] byte_count;
logic [5:0] preamble_count;
logic [7:0] test;

// assign test= {rx_clk,old,rx_clk && ~old};
// assign debug = {28'hBeefdea,end_of_frame,data_uncorrupted,destination_address==MAC_ADDRESS,1'b0};
    assign debug ={22'b0,preamble_count,state};


logic crc_input_valid;

logic [31:0] frame_check_sequence;


crc32 checksum
            (   .clk(clk),
                .rst(rst || (~crc_input_valid && ~end_of_frame)),
                .axiiv(crc_input_valid),
                .axiid({ether_rxd[0],ether_rxd[1],ether_rxd[2],ether_rxd[3]}),
                .axiod(frame_check_sequence),
                .rx_clk(rx_clk)
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
logic old;
// logic oldest;



always_ff @( posedge clk ) begin : stateMachine

    old <= rx_clk;
    if (rx_clk == 1 && old == 0)begin
    if (rst)begin
        
        state <= IDLE;
        byte_count<=0;
        bit_count<=0;
        preamble_count<=0;
        data<=0;
        curr_byte<=0;
        end_of_frame<=0;
        crc_input_valid<=0;
        source_address  <= 0;
        destination_address <= 0;
        

    end else begin
        
        case (state)
            IDLE: begin
                
              
                if (ether_crsdv ==1)begin
                    if(ether_rxd == 4'b0101)begin
                        if(preamble_count== 5'd13)begin
                            state<=SFD;
                            preamble_count<=0;
                            bit_count<=0;
                            byte_count<=0;
                            curr_byte<=0;
                            data<=0;
                            source_address  <= 0;
                            destination_address <= 0;
                        end else begin
                            preamble_count<=preamble_count+1;
                        end
                    end else begin
                        preamble_count<=0;
                        // state<=FAULTY;
                    end
                end else begin
                    preamble_count<=0;
                end

                if(end_of_frame)begin
                    end_of_frame<=0;
                    // debug<={28'hBeefdea,end_of_frame,data_uncorrupted,destination_address==MAC_ADDRESS,1'b0};
                    // debug<={frame_check_sequence};
                    
                end
                // debug<={28'hBeefdea,end_of_frame,data_uncorrupted,destination_address==MAC_ADDRESS,0};
              
            end
            SFD:begin
                curr_byte<={ether_rxd,curr_byte[7:4]};
                if(ether_crsdv ==1 )begin
                if(bit_count == 4)begin
                    // debug<={16'hBEEF,ether_rxd,curr_byte[7:4],bit_count,3'b1};
                    if({ether_rxd,curr_byte[7:4]} == SFD_CODE)begin
                        state<= DESTINATION;
                        crc_input_valid<=1;
                    end else begin
                        state <= FAULTY;
                    end
                    bit_count<=0;
                end else begin
                    bit_count<=bit_count+4;
                end
                end else begin
                    crc_input_valid<=0;
                    state=IDLE;
                end
            end
            DESTINATION:begin
                if(ether_crsdv ==1 )begin
                curr_byte<={ether_rxd,curr_byte[7:4]};
                if(bit_count == 4)begin
                    if(byte_count == 5)begin
                        if({data[39:0],ether_rxd,curr_byte[7:4]}== MAC_ADDRESS) begin
                            destination_address<={data[39:0],ether_rxd,curr_byte[7:4]};
                            state<=SOURCE;
                            byte_count<=0;
                        end else begin
                            state<=FAULTY;
                            destination_address<=0;
                            // destination_address<=32'hFEFEDEDE;
                        end
                    end else begin
                        byte_count<=byte_count+1;
                        data<={data[2392:0],ether_rxd,curr_byte[7:4]};
                        
                    end
                    bit_count<=0;
                end else begin
                    bit_count<=bit_count+4;
                end
                end else begin
                    crc_input_valid<=0;
                    state=IDLE;
                end
            end
            SOURCE:begin
                if(ether_crsdv ==1 )begin
                curr_byte<={ether_rxd,curr_byte[7:4]};
                if(bit_count == 4)begin
                    if(byte_count == 5)begin
                        source_address<={data[39:0],ether_rxd,curr_byte[7:4]};
                        state<=LENGTH;
                        byte_count<=0;
                    end else begin
                        byte_count<=byte_count+1;
                        data<={data[2392:0],ether_rxd,curr_byte[7:4]};  
                    end
                    bit_count<=0;
                end else begin
                    bit_count<=bit_count+4;
                end
                end else begin
                    crc_input_valid<=0;
                    state=IDLE;
                end
            end
            LENGTH:begin
                if(ether_crsdv ==1 )begin
                curr_byte<={ether_rxd,curr_byte[7:4]};
                if(bit_count == 4)begin
                    if(byte_count == 1)begin
                        data_length<={data[7:0],ether_rxd,curr_byte[7:4]};
                        if({data[7:0],ether_rxd,curr_byte[7:4]} > 300)begin
                            state<=FAULTY;
                        end else begin
                            state<=DATA_STREAM;
                        end
                        data<=0;
                        byte_count<=0;
                    end else begin
                        byte_count<=byte_count+1;
                        data<={data[2392:0],ether_rxd,curr_byte[7:4]};
                        
                    end
                    bit_count<=0;
                end else begin
                    bit_count<=bit_count+4;
                end
                end else begin
                    crc_input_valid<=0;
                    state=IDLE;
                end
            end
            DATA_STREAM:begin
                if(ether_crsdv ==1 )begin
                curr_byte<={ether_rxd,curr_byte[7:4]};
                if(bit_count == 4)begin
                    if(byte_count == data_length-1)begin
                        data<={data[2392:0],ether_rxd,curr_byte[7:4]};
                        state<=CRC;
                        byte_count<=0;
                    end else begin
                        byte_count<=byte_count+1;
                        data<={data[2392:0],ether_rxd,curr_byte[7:4]};
                    end
                    bit_count<=0;
                end else begin
                    bit_count<=bit_count+4;
                end
                end else begin
                    crc_input_valid<=0;
                    state=IDLE;
                end
            end
            CRC:begin
                if(ether_crsdv ==1 )begin
                if(bit_count == 4)begin
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
                    bit_count<=bit_count+4;
                    
                end
                end else begin
                    crc_input_valid<=0;
                    state=IDLE;
                end
            end
            FAULTY:begin
                if (ether_crsdv == 0) begin
                    state<= IDLE;
                    byte_count<=0;
                    crc_input_valid<=0;
                    

                end
                test<=ether_crsdv;
                
            end
  
        endcase



    end
    end
    
end


endmodule


`default_nettype wire