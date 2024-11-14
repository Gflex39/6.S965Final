`timescale 1ns / 1ps
`default_nettype none
module top_level
	(
		input wire clk,
        input wire rx_clk,
        input wire [3:0] rxd,
        input wire rx_ctrl,
        output wire tx_clk,
        output wire [3:0] txd,
        output wire tx_ctrl,

        input wire [3:0] btn,
        output logic [3:0] ss0_an,//anode control for upper four digits of seven-seg display
        output logic [3:0] ss1_an,//anode control for lower four digits of seven-seg display
        output logic [6:0] ss0_c, //cathode controls for the segments of upper four digits
        output logic [6:0] ss1_c//cathod controls for the segments of lower four digits
	);

    logic [6:0] ss_c;
    logic [11:0] counter;
    assign tx_clk = clk;

    always_ff @( posedge rx_clk) begin
        counter<=counter+1;
        
    end 
     
    seven_segment_controller mssc(.clk_in(clk),
                                  .rst_in(btn[0]),
                                  .val_in({counter,rxd}),
                                  .cat_out(ss_c),
                                  .an_out({ss0_an, ss1_an}));


    

    assign ss0_c = ss_c; //control upper four digit's cathodes!
    assign ss1_c = ss_c; //same as above but for lower four digits!              
endmodule