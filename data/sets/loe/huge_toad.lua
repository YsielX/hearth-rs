local card = {
    api_version = 1, id = "LOE_046", name = "Huge Toad",
    text = "<b>Deathrattle:</b> Deal 1 damage to a random enemy.",
    set = "LOE", type = "minion", rarity = "common", cost = 2, attack = 3, health = 2,
    tags = { "beast" }, keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local pool = {}
    for _, enemy in ipairs(ctx:enemy_characters(self)) do
        local dormant = false
        for _, keyword in ipairs(ctx:entity(enemy).keywords) do
            if keyword == "dormant" then dormant = true break end
        end
        if not dormant then pool[#pool + 1] = enemy end
    end
    if #pool > 0 then ctx:random_entity(pool, "deal_random_damage") end
end

function card.deal_random_damage(ctx, self, target) ctx:damage(target, 1) end
return card
