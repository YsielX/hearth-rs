return {
    api_version = 1,
    id = "AT_079",
    name = "Mysterious Challenger",
    text = "<b>Battlecry:</b> Put one of each <b>Secret</b> from your deck into the battlefield.",
    set = "TGT",
    type = "minion",
    class = "paladin",
    rarity = "epic",
    cost = 5,
    attack = 5,
    health = 5,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        local present, count = {}, #ctx:secrets(player)
        for _, secret in ipairs(ctx:secrets(player)) do present[ctx:entity(secret).card_id] = true end
        for _, entity in ipairs(ctx:deck(player)) do
            local definition = ctx:card_definition(ctx:entity(entity).card_id)
            local secret = definition.secret
            for _, keyword in ipairs(definition.keywords) do
                if keyword == "secret" then secret = true end
            end
            if count < 5 and secret and not present[definition.id] then
                present[definition.id] = true
                count = count + 1
                ctx:move(entity, "secret")
            end
        end
    end,
}
