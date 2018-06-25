extern crate pnet;
extern crate eui48;
extern crate pnet_datalink;

use std::env;
use std::io::{self, Write};
use std::net::{AddrParseError, Ipv4Addr};
use std::process;

use pnet_datalink::{Channel, NetworkInterface, ParseMacAddrErr};
use eui48::{MacAddress, Eui48};

use eui48::MacAddress::MacAddr;
use pnet::packet::arp::MutableArpPacket;
use pnet::packet::arp::ArpHardwareTypes;
use pnet::packet::ethernet::EtherTypes;
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::{MutablePacket, Packet};

use std::collections::HashMap;

fn send_packet(
    interface: NetworkInterface,
    source_mac: MacAddr,
    target_mac: MacAddr,
) {
    let (mut tx, _) = match pnet_datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type"),
        Err(e) => panic!("Error happened {}", e),
    };

    let mut ethernet_buffer = [0u8; 42];
    let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();

    ethernet_packet.set_destination(target_mac);
    ethernet_packet.set_source(source_mac);
    ethernet_packet.set_ethertype(EtherTypes::Arp);

    let mut arp_buffer = [0u8; 28];
    let mut arp_packet = MutableArpPacket::new(&mut arp_buffer).unwrap();

    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4);
    arp_packet.set_sender_hw_addr(source_mac);
    arp_packet.set_target_hw_addr(target_mac);

    ethernet_packet.set_payload(arp_packet.packet_mut());

    tx.send_to(ethernet_packet.packet(), Some(interface));
}

fn main() {
    let iface_name = match env::args().nth(1) {
        Some(n) => n,
        None => {
            writeln!(
                io::stderr(),
                "USAGE: packetdump <NETWORK INTERFACE> <SOURCE IP>"
            ).unwrap();
            process::exit(1);
        }
    };


    let interfaces = pnet_datalink::interfaces();
    let interfaces_name_match = |iface: &NetworkInterface| iface.name == iface_name;
    let interface = interfaces
        .into_iter()
        .filter(interfaces_name_match)
        .next()
        .unwrap();
    let source_mac = interface.mac_address();
    let bcast_mac = MacAddress::broadcast();
    // Create a new channel, dealing with layer 2 packets
    // let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
    //    Ok(Ethernet(tx, rx)) => (tx, rx),
    //     Ok(_) => panic!("Unhandled channel type"),
    //    Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    // };


    send_packet(
        interface,
        source_mac,
        bcast_mac,
    );

    println!("Sent ARP packet.");
}
