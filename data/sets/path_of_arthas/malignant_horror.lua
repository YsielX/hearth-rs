local card = {
    api_version = 1,
    id = "RLK_745",
    name = "Malignant Horror",
    text = "[x]<b>Reborn</b>\nAt the end of your turn,\nspend 4 <b>Corpses</b> to summon\na copy of this minion.",
    set = "PATH_OF_ARTHAS",
    type = "minion",
    class = "death_knight",
    rarity = "common",
    cost = 4,
    attack = 2,
    health = 4,
    tags = { "undead" },
    keywords = { "reborn" },
}

card.triggers = {{
    event = "turn_ended",
    timing = "after",
    active_zones = { "board" },
    condition = function(ctx, self, event)
        return event.player == ctx:controller(self)
    end,
    effect = function(ctx, self)
        local player = ctx:controller(self)
        if #ctx:board(player) < 7 then
            ctx:spend_resource_and_continue(player, "corpses", 4, 4, "summon_horror_copy")
        end
    end,
}}

function card.summon_horror_copy(ctx, self, spent)
    if spent > 0 then ctx:summon_copy(ctx:controller(self), self) end
end

return card
