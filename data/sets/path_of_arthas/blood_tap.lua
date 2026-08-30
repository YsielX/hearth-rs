local card = {
    api_version = 1,
    id = "RLK_712",
    name = "Blood Tap",
    text = "Give all minions in your hand +1/+1.\nSpend 2 <b>Corpses</b> to give them +1/+1 more.",
    set = "PATH_OF_ARTHAS",
    type = "spell",
    class = "death_knight",
    rarity = "rare",
    spell_school = "shadow",
    cost = 2,
    rune_cost = { blood = 1 },
}

function card.on_play(ctx, self)
    local player = ctx:controller(self)
    for _, entity in ipairs(ctx:hand(player)) do
        if ctx:entity(entity).type == "minion" then cardlib.effects.buff(ctx, entity, 1, 1) end
    end
    ctx:spend_resource_and_continue(player, "corpses", 2, 2, "buff_hand_again")
end

function card.buff_hand_again(ctx, self, spent)
    if spent == 0 then return end
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if ctx:entity(entity).type == "minion" then cardlib.effects.buff(ctx, entity, 1, 1) end
    end
end

return card
