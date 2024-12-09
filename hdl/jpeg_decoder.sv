module jpeg_decoder(
    input wire clk_in,
    input wire rst_in,
    input wire serial_in,
    input wire valid_in,

    output logic [63:0] row_out,
    output logic valid_out,
    output logic final_out
);
    logic [10:0]        value_mhd_med;
    logic [5:0]         run_mhd_med;
    logic [4:0]         size_mhd_med;
    logic               dc_mhd_med;
    logic               valid_mhd;

    logic signed [11:0] value_mhd_mzd;
    logic [5:0]         run_mhd_mzd;
    logic               valid_mhd_mzd;

    logic [95:0]        column_mzd_miq;
    logic               valid_mzd_miq;

    logic [95:0]        column_miq_midct;
    logic               valid_miq_midct;

    logic [11:0]        idct_in_0;
    logic [11:0]        idct_in_1;
    logic [11:0]        idct_in_2;
    logic [11:0]        idct_in_3;
    logic [11:0]        idct_in_4;
    logic [11:0]        idct_in_5;
    logic [11:0]        idct_in_6;
    logic [11:0]        idct_in_7;

    logic [7:0]         idct_out_0;
    logic [7:0]         idct_out_1;
    logic [7:0]         idct_out_2;
    logic [7:0]         idct_out_3;
    logic [7:0]         idct_out_4;
    logic [7:0]         idct_out_5;
    logic [7:0]         idct_out_6;
    logic [7:0]         idct_out_7;

    assign {idct_out_7, idct_out_6, idct_out_5, idct_out_4, idct_out_3, idct_out_2, idct_out_1, idct_out_0} = column_miq_midct;
    assign row_out = {idct_out_7, idct_out_6, idct_out_5, idct_out_4, idct_out_3, idct_out_2, idct_out_1, idct_out_0};

    huffman_decoder mhd (
        .clk_in(clk_in),
        .rst_in(rst_in),
        .serial_in(serial_in),
        .valid_in(valid_in),
        .value_out(value_mhd_mzd),
        .run_out(run_mhd_med),
        .size_out(size_mhd_med),
        .dc_out(dc_mhd_med),
        .valid_out(valid_mhd)
    );

    entropy_decoder med (
        .clk_in(clk_in),
        .rst_in(rst_in),
        .value_in(value_mhd_med),
        .run_in(run_mhd_med),
        .size_in(size_mhd_med),
        .valid_in(valid_mhd),
        .dc_in(dc_mhd_med),
        .value_out(value_mhd_mzd),
        .run_out(run_mhd_mzd),
        .valid_out(valid_mhd_mzd)
    );

    zigzag_decoder mzd (
        .clk_in(clk_in),
        .rst_in(rst_in),
        .value_in(value_mhd_mzd),
        .run_in(run_mhd_mzd),
        .valid_in(valid_mhd_mzd),
        .column_out(column_mzd_miq),
        .valid_out(valid_mzd_miq)
    );

    inverse_quantizer miq (
        .clk_in(clk_in),
        .rst_in(rst_in),
        .column_in(column_mzd_miq),
        .valid_in(valid_mzd_miq),
        .column_out(column_miq_midct),
        .valid_out(valid_miq_midct)
    );

    idct_2d midct (
        .rst_in(rst_in),
        .clk_in(clk_in),
        .idct_in_0(idct_in_0),
        .idct_in_1(idct_in_1),
        .idct_in_2(idct_in_2),
        .idct_in_3(idct_in_3),
        .idct_in_4(idct_in_4),
        .idct_in_5(idct_in_5),
        .idct_in_6(idct_in_6),
        .idct_in_7(idct_in_7),
        .valid_in(valid_miq_midct),
        .idct_out_0(idct_out_0),
        .idct_out_1(idct_out_1),
        .idct_out_2(idct_out_2),
        .idct_out_3(idct_out_3),
        .idct_out_4(idct_out_4),
        .idct_out_5(idct_out_5),
        .idct_out_6(idct_out_6),
        .idct_out_7(idct_out_7),
        .valid_out(valid_out),
        .final_out(final_out)
    );
endmodule