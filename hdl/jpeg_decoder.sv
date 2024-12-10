module jpeg_decoder (
    input wire clk_in,
    input wire rst_in,
    input wire serial_in,
    input wire valid_in,

    output logic [63:0] row_out,
    output logic valid_out,
    output logic final_out
);
  logic        [10:0] mhd_med_value;
  logic        [ 5:0] mhd_med_run;
  logic        [ 4:0] mhd_med_size;
  logic               mhd_med_dc;
  logic               mhd_med_valid;

  logic signed [11:0] med_mzd_value;
  logic        [ 5:0] med_mzd_run;
  logic               med_mzd_valid;

  logic        [95:0] mzd_miq_column;
  logic               mzd_miq_valid;

  logic        [95:0] miq_midct_column;
  logic               miq_midct_valid;

  logic        [11:0] idct_in_0;
  logic        [11:0] idct_in_1;
  logic        [11:0] idct_in_2;
  logic        [11:0] idct_in_3;
  logic        [11:0] idct_in_4;
  logic        [11:0] idct_in_5;
  logic        [11:0] idct_in_6;
  logic        [11:0] idct_in_7;

  logic        [ 7:0] idct_out_0;
  logic        [ 7:0] idct_out_1;
  logic        [ 7:0] idct_out_2;
  logic        [ 7:0] idct_out_3;
  logic        [ 7:0] idct_out_4;
  logic        [ 7:0] idct_out_5;
  logic        [ 7:0] idct_out_6;
  logic        [ 7:0] idct_out_7;

  assign {idct_in_7, idct_in_6, idct_in_5, idct_in_4, idct_in_3, idct_in_2, idct_in_1, idct_in_0} = miq_midct_column;
  assign row_out = {
    idct_out_7, idct_out_6, idct_out_5, idct_out_4, idct_out_3, idct_out_2, idct_out_1, idct_out_0
  };

  huffman_decoder mhd (
      .clk_in(clk_in),
      .rst_in(rst_in),
      .serial_in(serial_in),
      .valid_in(valid_in),
      .value_out(mhd_med_value),
      .run_out(mhd_med_run),
      .size_out(mhd_med_size),
      .dc_out(mhd_med_dc),
      .valid_out(mhd_med_valid)
  );

  entropy_decoder med (
      .clk_in(clk_in),
      .rst_in(rst_in),
      .value_in(mhd_med_value),
      .run_in(mhd_med_run),
      .size_in(mhd_med_size),
      .valid_in(mhd_med_valid),
      .dc_in(mhd_med_dc),
      .value_out(med_mzd_value),
      .run_out(med_mzd_run),
      .valid_out(med_mzd_valid)
  );

  zigzag_decoder mzd (
      .clk_in(clk_in),
      .rst_in(rst_in),
      .value_in(med_mzd_value),
      .run_in(med_mzd_run),
      .valid_in(med_mzd_valid),
      .column_out(mzd_miq_column),
      .valid_out(mzd_miq_valid)
  );

  inverse_quantizer miq (
      .clk_in(clk_in),
      .rst_in(rst_in),
      .column_in(mzd_miq_column),
      .valid_in(mzd_miq_valid),
      .column_out(miq_midct_column),
      .valid_out(miq_midct_valid)
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
      .valid_in(miq_midct_valid),
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
