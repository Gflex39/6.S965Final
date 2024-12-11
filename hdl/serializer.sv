`default_nettype none


module serializer(
    input wire clk,
    input wire rst,
    input wire [2400:0] data,
    input wire [32:0] length,
    input wire send,
    output logic valid,
    output logic output_data

);


enum logic [2:0] {SEND,IDLE } state;
logic [32:0] bits_to_send;
logic [2400:0] data_to_send;



initial state = IDLE;


always_ff @( posedge clk ) begin 
    if(rst)begin
        output_data<=0;
        bits_to_send<=0;
        data_to_send<=0;
        state<=IDLE;
        valid<=0;
    end else begin

        case (state)
            SEND: begin
                output_data<=data_to_send[0];
                bits_to_send<=bits_to_send-1;
                data_to_send<=data_to_send>>1;
                if(bits_to_send==1)begin
                    state<=IDLE;
                    valid<=0;
                end
            end
            IDLE:begin
                if(send)begin
                    valid<=1;
                    bits_to_send<=length;
                    data_to_send<=data;
                    state<=SEND;
                end
            end
        endcase

    end   
end

endmodule


`default_nettype wire