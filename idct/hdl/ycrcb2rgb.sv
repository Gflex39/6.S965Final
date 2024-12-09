`timescale 1ns / 10ps

module ycrcb2rgb (
    input  logic       clk_in,
    input  logic [9:0] y_in,
    input  logic [9:0] cr_in,
    input  logic [9:0] cb_in,
    output logic [9:0] r_out,
    output logic [9:0] g_out,
    output logic [9:0] b_out
);

  // Internal registers
  logic [9:0] dy, dcr, dcb;
  logic [22:0] ir, ig, ib;
  logic [19:0] rm;
  logic [19:0] gm1, gm2;
  logic [19:0] bm;

  // step 1: Calculate R, G, B
  //
  // Use N.M format for multiplication:
  // R = Y + 1.403Cr = Y + Cr + 0.403Cr
  // R = Y + Cr + 0x19C*Cr
  //
  // G = Y - 0.344Cb - 0.714Cr
  // G = Y - 0x160*Cb - 0x2DB*Cr
  //
  // B = Y + 1.770Cb = Y + Cb + 0.770Cb
  // B = Y + Cb + 0x314*Cb

  // delay y, cr and cb
  always_ff @(posedge clk_in) begin
    dy  <= y_in;
    dcr <= cr_in;
    dcb <= cb_in;
  end

  // calculate R
  always_ff @(posedge clk_in) begin
    rm <= 10'h19C * cr_in;
    ir <= ((dy + dcr) << 10) + rm;
  end

  // calculate G
  always_ff @(posedge clk_in) begin
    gm1 <= 10'h160 * cb_in;
    gm2 <= 10'h2DB * cr_in;
    ig  <= (dy << 10) - (gm1 + gm2);
  end

  // calculate B
  always_ff @(posedge clk_in) begin
    bm <= 10'h314 * cb_in;
    ib <= ((dy + dcb) << 10) + bm;
  end

  // step2: check boundaries
  always_ff @(posedge clk_in) begin
    // check R
    r_out <= (ir[19:10] & {10{!ir[22]}}) | {10{(!ir[22] && (ir[21] || ir[20]))}};

    // check G
    g_out <= (ig[19:10] & {10{!ig[22]}}) | {10{(!ig[22] && (ig[21] || ig[20]))}};

    // check B
    b_out <= (ib[19:10] & {10{!ib[22]}}) | {10{(!ib[22] && (ib[21] || ib[20]))}};
  end

endmodule
