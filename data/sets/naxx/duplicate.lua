local card = {
    api_version = 1,
    id = "FP1_018",
    name = "Duplicate",
    text = "<b>Secret:</b> When a friendly minion dies, put 2 copies of it into your hand.",
    set = "NAXX",
    type = "spell",
    class = "mage",
    rarity = "common",
    cost = 3,
    keywords = { "secret" },
    triggers = {
        {
            event = "entity_died",
            timing = "after",
            active_zones = { "secret" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and ctx:entity(event.entity).type == "minion"
            end,
            effect = function(ctx, self, event)
                local card_id = ctx:entity(event.entity).card_id
                ctx:reveal_secret(self)
                ctx:continue_with_value("duplicate_minion", card_id)
            end,
        },
    },
}

function card.duplicate_minion(ctx, self, card_id)
    -- Trigger collection for one death checkpoint observes a stable snapshot.
    -- This marker makes simultaneous deaths consume the Secret only once.
    if ctx:get_data(self, "triggered") == 1 then return end
    ctx:set_data(self, "triggered", 1)
    local player = ctx:controller(self)
    ctx:give_card(player, card_id)
    ctx:give_card(player, card_id)
end

return card
