local function minions_costing(ctx, cost)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.cost == cost then
            result[#result + 1] = card_id
        end
    end
    return result
end

local card = {
    api_version = 1,
    id = "LOE_086",
    name = "Summoning Stone",
    text = "Whenever you cast a spell, summon a random minion of the same Cost.",
    set = "LOE",
    type = "minion",
    rarity = "rare",
    cost = 5,
    attack = 0,
    health = 6,
    triggers = {
        {
            event = "mana_spent",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self) and event.player_cast
                    and ctx:entity(event.source).type == "spell"
            end,
            effect = function(ctx, self, event)
                ctx:set_data(self, "paid_spell", event.source)
                ctx:set_data(self, "paid_spell_cost", event.amount)
            end,
        },
        {
            event = "card_played",
            timing = "before",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and ctx:entity(event.entity).type == "spell"
            end,
            effect = function(ctx, self, event)
                -- A zero-Cost spell publishes no mana_spent event.
                if ctx:get_data(self, "paid_spell") ~= event.entity then
                    ctx:set_data(self, "paid_spell", event.entity)
                    ctx:set_data(self, "paid_spell_cost", 0)
                end
            end,
        },
        {
            event = "spell_cast",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self, event)
                local cost = ctx:entity(event.entity).cost
                if ctx:get_data(self, "paid_spell") == event.entity then
                    cost = ctx:get_data(self, "paid_spell_cost") or cost
                end
                local candidates = minions_costing(ctx, cost)
                if #candidates > 0 then
                    ctx:random_value(candidates, "summon_same_cost_minion")
                end
            end,
        },
    },
}

function card.summon_same_cost_minion(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
