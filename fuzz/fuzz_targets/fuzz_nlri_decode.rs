#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use prefixd::bgp::apipb::{
    FlowSpecComponent, FlowSpecComponentItem, FlowSpecIpPrefix,
    FlowSpecNlri as ProtoFlowSpecNlri, FlowSpecRule as ProtoFlowSpecRule, Nlri,
    flow_spec_rule, nlri,
};
use prefixd::bgp::GoBgpAnnouncer;

#[derive(Arbitrary, Debug)]
struct FuzzNlriInput {
    rules: Vec<FuzzRule>,
}

#[derive(Arbitrary, Debug)]
enum FuzzRule {
    IpPrefix {
        r#type: u32,
        prefix_len: u32,
        prefix: String,
        offset: u32,
    },
    Component {
        r#type: u32,
        items: Vec<FuzzItem>,
    },
}

#[derive(Arbitrary, Debug)]
struct FuzzItem {
    op: u32,
    value: u64,
}

fuzz_target!(|input: FuzzNlriInput| {
    let rules: Vec<ProtoFlowSpecRule> = input
        .rules
        .into_iter()
        .map(|r| match r {
            FuzzRule::IpPrefix {
                r#type,
                prefix_len,
                prefix,
                offset,
            } => ProtoFlowSpecRule {
                rule: Some(flow_spec_rule::Rule::IpPrefix(FlowSpecIpPrefix {
                    r#type,
                    prefix_len,
                    prefix,
                    offset,
                })),
            },
            FuzzRule::Component { r#type, items } => ProtoFlowSpecRule {
                rule: Some(flow_spec_rule::Rule::Component(FlowSpecComponent {
                    r#type,
                    items: items
                        .into_iter()
                        .map(|i| FlowSpecComponentItem {
                            op: i.op,
                            value: i.value,
                        })
                        .collect(),
                })),
            },
        })
        .collect();

    let proto_nlri = ProtoFlowSpecNlri { rules };
    let wrapper = Nlri {
        nlri: Some(nlri::Nlri::FlowSpec(proto_nlri)),
    };

    let announcer = GoBgpAnnouncer::new("127.0.0.1:50051".to_string());
    let _ = announcer.decode_flowspec_nlri(&wrapper);
});
