// (c) 2017 KAI OS TECHNOLOGIES (HONG KONG) LIMITED All rights reserved. This
// file or any portion thereof may not be reproduced or used in any manner
// whatsoever without the express written permission of KAI OS TECHNOLOGIES
// (HONG KONG) LIMITED. KaiOS is the trademark of KAI OS TECHNOLOGIES (HONG KONG)
// LIMITED or its affiliate company and may be registered in some jurisdictions.
// All other trademarks are the property of their respective owners.

/// The messages exchanged among internal threads using the message broker.
use frame_messages::{ClientPayload, FilterAck, FilterFrame};
use socket_relay::SocketRelay;

#[derive(Clone, Debug)]
pub enum InternalMessage {
    NewClientMessage(ClientPayload),
    RelayReady(SocketRelay),
    NewFilter(FilterFrame),
    FilterAck(FilterAck),
    Shutdown,
}
