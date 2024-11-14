module cocotb_iverilog_dump();
initial begin
    $dumpfile("/Users/seblohier/6.S965/finalproject/6.S965Final/sim/sim_build/ether.fst");
    $dumpvars(0, ether);
end
endmodule
