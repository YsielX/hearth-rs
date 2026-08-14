local candle = {
    id = "LOOT_526t",
    name = "Darkness Candle",
    text = "<b>Casts When Drawn</b>\nSnuff out a candle.",
    set = "LOOTAPALOOZA",
    type = "spell",
    collectible = false,
    cost = 4,
    triggers = {
        {
            event = "card_drawn",
            timing = "after",
            active_zones = { "hand" },
            condition = function(_, self, event) return event.entity == self end,
            effect = function(ctx, self)
                local darkness = ctx:get_data(self, "darkness_source")
                ctx:cast_drawn(self)
                if darkness == 0 or ctx:entity(darkness).card_id ~= "LOOT_526d" then return end
                local count = ctx:get_data(darkness, "candles_drawn") + 1
                ctx:set_data(darkness, "candles_drawn", count)
                if count == 3 then
                    ctx:transform_preserving_scripts(darkness, "LOOT_526")
                end
            end,
        },
    },
}

local dormant_darkness = {
    id = "LOOT_526d",
    name = "The Darkness",
    text = "",
    set = "LOOTAPALOOZA",
    type = "minion",
    rarity = "legendary",
    collectible = false,
    cost = 4,
    attack = 20,
    health = 20,
    keywords = { "dormant" },
    rules = {
        can_be_targeted = function() return false end,
        can_be_attacked = function() return false end,
        can_be_destroyed = function() return false end,
        can_be_silenced = function() return false end,
    },
}

local card = {
    api_version = 1,
    id = "LOOT_526",
    name = "The Darkness",
    text = "[x]Starts <b>Dormant</b>.\n<b>Battlecry:</b> Shuffle 3 Candles\ninto the enemy deck. When\ndrawn, this awakens.",
    set = "LOOTAPALOOZA",
    type = "minion",
    rarity = "legendary",
    cost = 4,
    attack = 20,
    health = 20,
    keywords = { "battlecry" },
    tokens = { candle, dormant_darkness },
    triggers = {
        {
            event = "minion_summoned",
            timing = "after",
            active_zones = { "board" },
            condition = function(_, self, event) return event.entity == self end,
            effect = function(ctx, self)
                ctx:transform_preserving_scripts(self, "LOOT_526d")
            end,
        },
        {
            event = "card_created",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.source == self and ctx:entity(event.entity).card_id == "LOOT_526t"
            end,
            effect = function(ctx, self, event)
                ctx:set_data(event.entity, "darkness_source", self)
            end,
        },
    },
}

function card.on_battlecry(ctx, self)
    local enemy = ctx:opponent(ctx:controller(self))
    ctx:shuffle_card_into_deck(enemy, "LOOT_526t")
    ctx:shuffle_card_into_deck(enemy, "LOOT_526t")
    ctx:shuffle_card_into_deck(enemy, "LOOT_526t")
    ctx:transform_preserving_scripts(self, "LOOT_526d")
end

return card
