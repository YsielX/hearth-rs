local function has_keyword(ctx, entity, wanted)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do if keyword == wanted then return true end end
    return false
end
local card = {
    api_version = 1, id = "UNG_940", name = "Awaken the Makers",
    text = "<b>Quest:</b> Summon\n6 <b>Deathrattle</b> minions.<b>\nReward:</b> Amara, Warden of Hope.",
    set = "UNGORO", type = "spell", class = "priest", rarity = "legendary", cost = 1,
    keywords = { "quest" },
}
card.triggers = {{
    event = "minion_summoned", timing = "after", active_zones = { "secret" },
    condition = function(ctx, self, event)
        return event.player == ctx:controller(self) and ctx:get_data(self, "completed") == 0
            and has_keyword(ctx, event.entity, "deathrattle")
    end,
    effect = function(ctx, self)
        local progress = ctx:get_data(self, "quest_progress") + 1
        ctx:set_data(self, "quest_progress", progress)
        if progress >= 6 then
            ctx:set_data(self, "completed", 1)
            ctx:reveal_secret(self)
            cardlib.effects.give_card(ctx, ctx:controller(self), "UNG_940t8")
        end
    end,
}}
card.tokens = {{
    id = "UNG_940t8", name = "Amara, Warden of Hope",
    text = "[x]<b>Taunt</b>\n<b>Battlecry:</b> Set your\nhero's Health to 40.",
    set = "UNGORO", type = "minion", class = "priest", cost = 5, attack = 8, health = 8,
    keywords = { "taunt", "battlecry" },
    on_battlecry = function(ctx, self)
        local hero = ctx:player(ctx:controller(self)).hero
        cardlib.effects.modify(ctx, hero, { stat = "health", operation = "final_set", value = 40, silenciable = false })
        ctx:continue_with_entity("set_amara_health", hero)
    end,
    set_amara_health = function(ctx, self, hero) ctx:set_health(hero, 40) end,
}}
return card
