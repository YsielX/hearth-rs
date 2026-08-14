local card = {
    api_version = 1,
    id = "FP1_004",
    name = "Mad Scientist",
    text = "<b>Deathrattle:</b> Put a <b>Secret</b> from your deck into the battlefield.",
    set = "NAXX",
    type = "minion",
    rarity = "common",
    cost = 2,
    attack = 2,
    health = 2,
    tags = { "undead" },
    keywords = { "deathrattle" },
}

local function has_keyword(definition, wanted)
    for _, keyword in ipairs(definition.keywords) do
        if keyword == wanted then return true end
    end
    return false
end

function card.on_deathrattle(ctx, self)
    local player = ctx:controller(self)
    local secrets = {}
    for _, entity in ipairs(ctx:deck(player)) do
        local definition = ctx:card_definition(ctx:entity(entity).card_id)
        if has_keyword(definition, "secret") then
            secrets[#secrets + 1] = entity
        end
    end
    if #secrets > 0 then ctx:random_entity(secrets, "deploy_secret") end
end

function card.deploy_secret(ctx, self, secret)
    ctx:move(secret, "secret")
end

return card
