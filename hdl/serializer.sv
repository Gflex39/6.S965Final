`default_nettype none


module serializer(
    input wire clk,
    input wire rst,
    input wire [2400:0] data,
    input wire [32:0] length,
    input wire send,
    input wire valid,
    output logic output_data

);


enum logic [2:0] {SEND,IDLE } state;
logic [32:0] bits_to_send;
logic [2400:0] data_to_send;


assign valid = state == SEND;


always_ff @( posedge clk ) begin 
    if(rst)begin
        output_data<=0;
        bits_to_send<=0;
        data_to_send<=0;
        state<=IDLE;
    end else begin

        casex (state)
            SEND: begin
                output_data<=data_to_send[0];
                bits_to_send<=bits_to_send-1;
                data_to_send<=data_to_send>>1;
                if(bits_to_send==0)begin
                    state<=IDLE;
                end
            end
            IDLE:begin
                if(send)begin
                    bits_to_send<=length-1;
                    data_to_send<=(data)>>1;
                    output_data<=data[0];
                    state<=SEND;
                end
            end
            default: 
        endcase

    end   
end

endmodule


`default_nettype wire