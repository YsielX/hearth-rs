local spare_parts = {
    "PART_001", "PART_002", "PART_003", "PART_004",
    "PART_005", "PART_006", "PART_007",
}

local card = {
    api_version = 1,
    id = "GVG_034",
    name = "Mech-Bear-Cat",
    text = "Whenever this minion takes damage, add a <b>Spare Part</b> card to your hand.",
    set = "GVG",
    type = "minion",
    class = "druid",
    rarity = "rare",
    cost = 6,
    attack = 7,
    health = 6,
    tags = { "mech", "beast" },
    triggers = {
        {
            event = "damaged",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.target == self and event.amount > 0
            end,
            effect = function(ctx, self)
                ctx:random_value(spare_parts, "receive_spare_part")
            end,
        },
    },
}

function card.receive_spare_part(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end

return card
