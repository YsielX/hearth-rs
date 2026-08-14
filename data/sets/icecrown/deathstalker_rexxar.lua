local card = {
    api_version = 1,
    id = "ICC_828",
    name = "Deathstalker Rexxar",
    text = "[x]<b>Battlecry:</b> Deal 2 damage\nto all enemy minions.",
    set = "ICECROWN",
    type = "hero",
    class = "hunter",
    cost = 6,
    health = 30,
    armor = 5,
    hero_power = "ICC_828p",
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local targets = {}
    for _, minion in ipairs(ctx:minions()) do
        if ctx:controller(minion) ~= ctx:controller(self) then
            targets[#targets + 1] = minion
        end
    end
    ctx:damage_all(targets, 2)
end

return card
