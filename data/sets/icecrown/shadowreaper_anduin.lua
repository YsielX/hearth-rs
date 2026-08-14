local card = {
    api_version = 1,
    id = "ICC_830",
    name = "Shadowreaper Anduin",
    text = "<b>Battlecry:</b> Destroy all minions with 5 or more Attack.",
    set = "ICECROWN",
    type = "hero",
    class = "priest",
    cost = 8,
    health = 30,
    armor = 5,
    hero_power = "ICC_830p",
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    for _, minion in ipairs(ctx:minions()) do
        if ctx:entity(minion).attack >= 5 then ctx:destroy(minion) end
    end
end

return card
