local card = {
    api_version = 1,
    id = "CFM_616",
    name = "Pilfered Power",
    text = "Gain an empty Mana Crystal for each friendly minion.",
    set = "GANGS",
    type = "spell",
    class = "druid",
    rarity = "epic",
    cost = 3,
}

function card.on_play(ctx, self)
    local player = ctx:controller(self)
    local count = 0
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        local dormant = false
        for _, keyword in ipairs(ctx:entity(minion).keywords) do
            if keyword == "dormant" then dormant = true break end
        end
        if not dormant then count = count + 1 end
    end
    if count == 0 then return end
    if ctx:player(player).max_mana >= 10 or ctx:player(player).mana >= 10 then
        cardlib.effects.give_card(ctx, player, "CS2_013t")
    else
        ctx:gain_mana_crystals(player, count, false)
    end
end

return card
