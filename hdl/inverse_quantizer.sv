module inverse_quantizer(
    input wire clk_in,
    input wire rst_in,
    input wire [95:0] column_in,
    input wire valid_in,

    output logic [95:0] column_out,
    output logic valid_out
);
    logic [2:0] counter;

    logic signed [11:0] c [7:0];
    logic signed [11:0] o [7:0];

    assign c[0] = column_in[11: 0];
    assign c[1] = column_in[23:12];
    assign c[2] = column_in[35:24];
    assign c[3] = column_in[47:36];
    assign c[4] = column_in[59:48];
    assign c[5] = column_in[71:60];
    assign c[6] = column_in[83:72];
    assign c[7] = column_in[95:84];

    assign column_out[11: 0] = o[0];
    assign column_out[23:12] = o[1];
    assign column_out[35:24] = o[2];
    assign column_out[47:36] = o[3];
    assign column_out[59:48] = o[4];
    assign column_out[71:60] = o[5];
    assign column_out[83:72] = o[6];
    assign column_out[95:84] = o[7];

    always_ff @(posedge clk_in) begin
        if (rst_in) begin
            valid_out <= 0;
            for (integer i = 0; i < 8; i = i + 1) begin
                o[i] <= 0;
            end
        end else if (valid_in) begin
            counter <= counter + 1;
            valid_out <= 1;
            case (counter)
                3'd0: begin
                    o[0] <= (c[0]<<<4);
                    o[1] <= (c[1]<<<3)+(c[1]<<<2);
                    o[2] <= (c[2]<<<3)+(c[2]<<<2)+(c[2]<<<1);
                    o[3] <= (c[3]<<<3)+(c[3]<<<2)+(c[2]<<<1);
                    o[4] <= (c[4]<<<4)+(c[4]<<<1);
                    o[5] <= (c[5]<<<4)+(c[5]<<<3);
                    o[6] <= (c[6]<<<5)+(c[6]<<<4)+c[6];
                    o[7] <= (c[7]<<<6)+(c[7]<<<3);
                end
                3'd1: begin
                    o[0] <= (c[0]<<<3)+(c[0]<<<1)+c[0];
                    o[1] <= (c[1]<<<3)+(c[1]<<<2);
                    o[2] <= (c[2]<<<3)+(c[2]<<<2)+c[2];
                    o[3] <= (c[3]<<<4)+c[3];
                    o[4] <= (c[4]<<<4)+(c[4]<<<2)+(c[4]<<<1);
                    o[5] <= (c[5]<<<5)+(c[5]<<<1)+c[5]; // c[5] * 35;
                    o[6] <= (c[6]<<<6);
                    o[7] <= c[7] * 92;
                end
                3'd2: begin
                    o[0] <= c[0] * 10;
                    o[1] <= c[1] * 14;
                    o[2] <= c[2] * 16;
                    o[3] <= c[3] * 22;
                    o[4] <= c[4] * 37;
                    o[5] <= c[5] * 55;
                    o[6] <= c[6] * 78;
                    o[7] <= c[7] * 95;
                end
                3'd3: begin
                    o[0] <= c[0] * 16;
                    o[1] <= c[1] * 19;
                    o[2] <= c[2] * 24;
                    o[3] <= c[3] * 29;
                    o[4] <= c[4] * 56;
                    o[5] <= c[5] * 64;
                    o[6] <= c[6] * 87;
                    o[7] <= c[7] * 98;
                end
                3'd4: begin
                    o[0] <= c[0] * 24;
                    o[1] <= c[1] * 26;
                    o[2] <= c[2] * 40;
                    o[3] <= c[3] * 51;
                    o[4] <= c[4] * 68;
                    o[5] <= c[5] * 81;
                    o[6] <= c[6] * 103;
                    o[7] <= c[7] * 112;
                end
                3'd5: begin
                    o[0] <= c[0] * 40;
                    o[1] <= c[1] * 58;
                    o[2] <= c[2] * 57;
                    o[3] <= c[3] * 87;
                    o[4] <= c[4] * 109;
                    o[5] <= c[5] * 104;
                    o[6] <= c[6] * 121;
                    o[7] <= c[7] * 100;
                end
                3'd6: begin
                    o[0] <= c[0] * 51;
                    o[1] <= c[1] * 60;
                    o[2] <= c[2] * 69;
                    o[3] <= c[3] * 80;
                    o[4] <= c[4] * 103;
                    o[5] <= c[5] * 113;
                    o[6] <= c[6] * 120;
                    o[7] <= c[7] * 103;
                end
                3'd7: begin
                    o[0] <= c[0] * 61;
                    o[1] <= c[1] * 55;
                    o[2] <= c[2] * 56;
                    o[3] <= c[3] * 62;
                    o[4] <= c[4] * 77;
                    o[5] <= c[5] * 92;
                    o[6] <= c[6] * 101;
                    o[7] <= c[7] * 99;
                end
            endcase
        end else begin
            valid_out <= 0;
        end
    end

endmodule