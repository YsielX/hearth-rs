local card = {
    api_version = 1, id = "OG_085", name = "Demented Frostcaller",
    text = "After you cast a spell, <b>Freeze</b> a random enemy.",
    set = "OG", type = "minion", class = "mage", rarity = "rare",
    cost = 4, attack = 2, health = 4,
}
card.triggers = {{
    event = "spell_cast", timing = "after", active_zones = { "board" },
    condition = function(ctx, self, event)
        return event.player == ctx:controller(self) and event.player_cast
    end,
    effect = function(ctx, self)
        local pool = {}
        for _, enemy in ipairs(ctx:enemy_characters(self)) do
            local dormant = false
            for _, keyword in ipairs(ctx:entity(enemy).keywords) do
                if keyword == "dormant" then dormant = true break end
            end
            if not dormant then pool[#pool + 1] = enemy end
        end
        if #pool > 0 then ctx:random_entity(pool, "freeze_enemy") end
    end,
}}
function card.freeze_enemy(ctx, self, target) ctx:freeze(target) end
return card
