problem: writing to TUN apparently fails with various odd errors. 

Solution: instead of hacking rawfd, use the crate https://github.com/yaa110/tokio-tun
