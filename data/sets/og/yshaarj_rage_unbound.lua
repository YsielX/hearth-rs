local card = {
    api_version = 1, id = "OG_042", name = "Y'Shaarj, Rage Unbound",
    text = "At the end of your turn, put a minion from your deck into the battlefield.",
    set = "OG", type = "minion", rarity = "legendary", cost = 10, attack = 10, health = 10,
}
card.triggers = {{
    event = "turn_ended", timing = "after", active_zones = { "board" },
    condition = function(ctx, self, event)
        return event.player == ctx:controller(self) and #ctx:board(event.player) < 7
    end,
    effect = function(ctx, self)
        local pool = {}
        for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
            if ctx:entity(entity).type == "minion" then pool[#pool + 1] = entity end
        end
        if #pool > 0 then ctx:random_entity(pool, "recruit_minion") end
    end,
}}
function card.recruit_minion(ctx, self, entity) ctx:recruit(ctx:controller(self), entity) end
return card
