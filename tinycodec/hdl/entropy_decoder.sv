`timescale 1ns / 1ps
`default_nettype none

module entropy_decoder#(
    parameter DELTA_ENCODE = 1
)
(
    input wire          clk_in,
    input wire          rst_in,
    input wire [10:0]   value_in,
    input wire [4:0]    run_in,
    input wire [4:0]    size_in,
    input wire          dc_in,

    output logic signed [11:0]  value_out,
    output logic [4:0]          run_out,
    output logic                dc_out,
    output logic                valid_out
);
endmodule