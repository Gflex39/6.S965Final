module negedge_detector(
    input wire clk_in,
    input wire rst_in,
    input wire level_in,
    output logic level_out
);

    logic level1;

    pipeline#(.PIPELINE_STAGES(1), .PIPELINE_WIDTH(1)) p1(.clk_in(clk_in), .rst_in(rst_in), .signal_in(level_in), .signal_out(level1));

    assign level_out = {level_in, level1} == 2'b01;
endmodule
