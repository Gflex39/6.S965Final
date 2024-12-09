module cocotb_iverilog_dump();
initial begin
    $dumpfile("/Users/seblohier/6.S965/finalproject/6.S965Final/sim/sim_build/ether_4.fst");
    $dumpvars(0, ether_4);
end
endmodule
